//! Shards cluster implementation.

use std::sync::Arc;

use anyhow::Context;
use futures::StreamExt;
use raidprotect_model::{
    cache::{http::CacheHttp, RedisClient},
    config::BotConfig,
    database::DbClient,
};
use tracing::{info, info_span, instrument, trace};
use twilight_gateway::{cluster::Events, Cluster, Intents};
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
/// This type is a wrapper around twilight [`Cluster`] and manages incoming
/// events from Discord.
#[derive(Debug)]
pub struct ShardCluster {
    /// Inner shard cluster managed by twilight
    cluster: Arc<Cluster>,
    /// Events stream
    events: Events,
    /// Shared cluster state
    state: Arc<ClusterState>,
}

impl ShardCluster {
    /// Initialize a new [`ShardCluster`].
    ///
    /// This method also initialize an [`HttpClient`] and a [`RedisClient`],
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

        let redis = RedisClient::new(&config.database.redis_uri).await?;
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

        let intents = Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT;

        let (cluster, events) = Cluster::builder(config.token, intents)
            .http_client(http.clone())
            .presence(presence())
            .build()
            .await?;

        info!("started cluster with {} shards", cluster.shards().len());

        let state = ClusterState::new(redis, mongodb, http, current_user);

        register_commands(&state, application.id).await;

        Ok(Self {
            cluster: Arc::new(cluster),
            events,
            state: Arc::new(state),
        })
    }

    /// Start the cluster and handle incoming events.
    ///
    /// A [`ShutdownSubscriber`] must be provided to gracefully stop the cluster.
    #[instrument(name = "start_cluster", skip_all)]
    pub async fn start(mut self, mut shutdown: ShutdownSubscriber) {
        // Start the cluster
        let cluster = self.cluster.clone();
        tokio::spawn(async move {
            cluster.up().await;
        });

        // Handle incoming events
        tokio::select! {
            _ = self.handle_events() => {},
            _ = shutdown.wait_shutdown() => {},
        };

        self.cluster.down();
    }

    /// Handle incoming events
    async fn handle_events(&mut self) {
        while let Some((_shard_id, event)) = self.events.next().await {
            let span = info_span!("handle_event");

            span.in_scope(|| {
                trace!(event = ?event, "received event");

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

/// Current state of the cluster.
///
/// This type hold shared types such as the cache or the http client. It does
/// not implement [`Clone`] and is intended to be wrapped inside a [`Arc`].
#[derive(Debug)]
pub struct ClusterState {
    /// In-memory cache
    redis: RedisClient,
    /// MongoDB client
    mongodb: DbClient,
    /// Http client
    http: Arc<HttpClient>,
    /// Bot user id
    current_user: Id<ApplicationMarker>,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(
        redis: RedisClient,
        mongodb: DbClient,
        http: Arc<HttpClient>,
        current_user: Id<ApplicationMarker>,
    ) -> Self {
        Self {
            redis,
            mongodb,
            http,
            current_user,
        }
    }

    /// Get the cluster [`RedisClient`].
    pub fn redis(&self) -> &RedisClient {
        &self.redis
    }

    /// Get the cluster [`DbClient`].
    pub fn mongodb(&self) -> &DbClient {
        &self.mongodb
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the cluster [`CacheHttp`]
    pub fn cache_http(&self, guild_id: Id<GuildMarker>) -> CacheHttp {
        self.redis.http(&self.http, guild_id)
    }

    /// Get the bot user id
    pub fn current_user(&self) -> Id<ApplicationMarker> {
        self.current_user
    }
}
