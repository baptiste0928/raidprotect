//! Shards cluster implementation.

use std::sync::Arc;

use futures::StreamExt;
use raidprotect_cache::redis::{RedisClient, RedisClientError};
use raidprotect_interaction::register_commands;
use raidprotect_model::mongodb::{MongoDbClient, MongoDbError};
use raidprotect_state::ClusterState;
use raidprotect_util::shutdown::ShutdownSubscriber;
use thiserror::Error;
use tracing::{info, info_span, instrument, trace};
use twilight_gateway::{
    cluster::{ClusterStartError, Events, ShardScheme},
    Cluster, Intents,
};
use twilight_http::{response::DeserializeBodyError, Client as HttpClient, Error as HttpError};
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
};

use crate::{config::Config, event::ProcessEvent};

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
    pub async fn new(config: Config) -> Result<Self, ClusterError> {
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

        let redis = RedisClient::new(&config.redis_uri).await?;
        redis.ping().await?; // Ensure redis is reachable

        let mongodb = MongoDbClient::connect(&config.mongodb_uri, config.mongodb_database).await?;
        mongodb.ping().await?; // Ensure database is reachable

        let intents = Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT;

        let (cluster, events) = Cluster::builder(config.token, intents)
            .http_client(http.clone())
            .shard_scheme(ShardScheme::Auto)
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

/// Error when initializing a [`ShardCluster`].
#[derive(Debug, Error)]
pub enum ClusterError {
    /// HTTP request failed
    #[error("http error: {0}")]
    Http(#[from] HttpError),
    /// Response body deserialization error
    #[error("deserialize error: {0}")]
    Deserialize(#[from] DeserializeBodyError),
    /// Failed to start cluster
    #[error("failed to start cluster: {0}")]
    Start(#[from] ClusterStartError),
    /// Error while connecting to Redis instance
    #[error("failed to connect to Redis")]
    Redis(#[from] RedisClientError),
    /// Error while connecting to MongoDB database
    #[error("failed to connect to MongoDB")]
    MongoDb(#[from] MongoDbError),
}
