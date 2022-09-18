//! Shards cluster implementation.

use std::sync::Arc;

use anyhow::Context;
use futures_util::StreamExt;
use raidprotect_model::{
    cache::{discord::http::CacheHttp, CacheClient},
    config::BotConfig,
    database::DbClient,
};
use tracing::{debug, error, info, info_span, instrument, trace, warn};
use twilight_gateway::{
    message::CloseFrame,
    stream::{self, ShardEventStream},
    Config, Intents, Shard,
};
use twilight_http::Client as HttpClient;
use twilight_model::{
    gateway::{
        payload::outgoing::update_presence::UpdatePresencePayload,
        presence::{ActivityType, MinimalActivity, Status},
    },
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};

use crate::{
    event::ProcessEvent, interaction::register_commands, util::shutdown::ShutdownSubscriber,
};

/// Discord shards cluster.
///
/// This type is a wrapper around twilight [`Shard`]s and manages incoming
/// events from Discord.
#[derive(Debug)]
pub struct BotShards {
    /// Started bot shards.
    shards: Vec<Shard>,
    /// Shared bot state
    state: BotState,
}

impl BotShards {
    /// Initialize a new [`BotShards`].
    ///
    /// This method also initialize an [`HttpClient`] and a [`CacheClient`],
    /// that can be later retrieved using corresponding methods.
    pub async fn new(config: BotConfig) -> Result<Self, anyhow::Error> {
        // Initialize HTTP client and get current user.
        let http = Arc::new(HttpClient::new(config.token.clone()));
        let application = http
            .current_user_application()
            .exec()
            .await?
            .model()
            .await?;
        let current_user = application.id;

        info!("logged as {} with ID {}", application.name, current_user);

        // Initialize database connections.
        let redis = CacheClient::connect(&config.database.redis_uri).await?;
        redis.ping().await.context("failed to connect to redis")?;

        let mongodb = DbClient::connect(
            &config.database.mongodb_uri,
            config.database.mongodb_database,
        )
        .await?;
        mongodb
            .ping()
            .await
            .context("failed to connect to mongodb")?;

        // Start bot shards.
        let intents = Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT;

        let config = Config::builder(config.token, intents)
            .presence(presence())
            .build();
        let per_shard_config = |_| config.clone();

        info!("starting cluster ...");

        let mut shards = Vec::new();
        let mut start_stream = stream::start_recommended(&http, per_shard_config)
            .await
            .context("failed to fetch recommended shard configuration")?;

        while let Some(shard_result) = start_stream.next().await {
            let shard = shard_result.context("failed to start shard")?;
            debug!("shard {} started", shard.id());

            shards.push(shard);
        }

        info!("started bot with {} shards", shards.len());

        let state = BotState::new(redis, mongodb, http, current_user);

        register_commands(&state, application.id).await;

        Ok(Self { shards, state })
    }

    /// Handle incoming from Discord.
    ///
    /// A [`ShutdownSubscriber`] must be provided to gracefully stop the bot.
    #[instrument(name = "start_cluster", skip_all)]
    pub async fn handle(mut self, mut shutdown: ShutdownSubscriber) {
        tokio::select! {
            _ = self.handle_events_inner() => {},
            _ = shutdown.wait_shutdown() => {},
        };

        info!("shutting down shards ...");
        for mut shard in self.shards {
            if let Err(error) = shard.close(CloseFrame::NORMAL).await {
                warn!(error = ?error, "failed to close shard {}", shard.id());
            }
        }
    }

    async fn handle_events_inner(&mut self) {
        let mut stream = ShardEventStream::new(self.shards.iter_mut());

        loop {
            let (shard, event) = match stream.next().await {
                Some((shard, Ok(event))) => (shard, event),
                Some((shard, Err(error))) => {
                    if error.is_fatal() {
                        error!(?error, shard = ?shard.id(), "fatal error while receiving error, shutting down");

                        break;
                    } else {
                        warn!(?error, shard = ?shard.id(), "error while receiving event");

                        continue;
                    }
                }
                None => break,
            };

            // Process event.
            let span = info_span!("handle_event");

            span.in_scope(|| {
                trace!(event = ?event, shard = ?shard.id(), "received event");

                let state = self.state.clone();
                tokio::spawn(event.process(state));
            });
        }
    }
}

/// Get the bot presence.
fn presence() -> UpdatePresencePayload {
    let activity = MinimalActivity {
        kind: ActivityType::Watching,
        name: String::from("raidprotect.org"),
        url: None,
    };

    UpdatePresencePayload {
        activities: vec![activity.into()],
        afk: false,
        since: None,
        status: Status::Online,
    }
}

/// Current state of the bot.
///
/// This type hold shared types such as the cache or the http client. It implement
/// [`Clone`] since all underlying types are wrapped in an [`Arc`].
#[derive(Debug, Clone)]
pub struct BotState {
    pub cache: CacheClient,
    pub database: DbClient,
    pub http: Arc<HttpClient>,
    pub current_user: Id<ApplicationMarker>,
}

impl BotState {
    /// Initialize a new [`BotState`].
    pub fn new(
        cache: CacheClient,
        mongodb: DbClient,
        http: Arc<HttpClient>,
        current_user: Id<ApplicationMarker>,
    ) -> Self {
        Self {
            cache,
            database: mongodb,
            http,
            current_user,
        }
    }

    /// Get the [`CacheHttp`] client associated with the cache client.
    pub fn cache_http(&self, guild_id: Id<GuildMarker>) -> CacheHttp {
        self.cache.http(&self.http, guild_id)
    }
}
