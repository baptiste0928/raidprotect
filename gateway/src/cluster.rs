//! Shards cluster implementation.

use std::sync::Arc;

use futures::StreamExt;
use raidprotect_cache::InMemoryCache;
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

use crate::event::ProcessEvent;

/// Current state of the cluster.
///
/// This type hold shared types such as the cache or the http client. It does
/// not implement [`Clone`] and is intended to be wrapped inside a [`Arc`].
#[derive(Debug)]
pub struct ClusterState {
    /// In-memory cache
    cache: InMemoryCache,
    /// Http client
    http: Arc<HttpClient>,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    fn new(cache: InMemoryCache, http: Arc<HttpClient>) -> Self {
        Self { cache, http }
    }

    /// Get the cluster [`InMemoryCache`].
    pub fn cache(&self) -> &InMemoryCache {
        &self.cache
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> &HttpClient {
        &self.http
    }
}

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
    /// This method also initialize an [`HttpClient`] and an [`InMemoryCache`],
    /// that can be later retrieved using corresponding methods.
    pub async fn new(token: String) -> Result<Self, ClusterError> {
        // Initialize HTTP client and get current user.
        let http = Arc::new(HttpClient::new(token.clone()));
        let current_user = http.current_user().exec().await?.model().await?;

        info!(
            "Logged as {} with ID {}",
            current_user.name, current_user.id
        );

        let cache = InMemoryCache::new(current_user.id);

        let intents = Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_MESSAGES;

        let (cluster, events) = Cluster::builder(token.to_string(), intents)
            .http_client(http.clone())
            .shard_scheme(ShardScheme::Auto)
            .presence(presence())
            .build()
            .await?;

        info!("Started cluster with {} shards", cluster.shards().len());

        let state = ClusterState::new(cache, http);

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
                event.process(self.state.clone());
            });
        }
    }

    /// Get the current [`ClusterState`].
    pub fn state(&self) -> Arc<ClusterState> {
        self.state.clone()
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
}
