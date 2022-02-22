//! Shards cluster implementation.

use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
};

use futures::StreamExt;
use raidprotect_cache::InMemoryCache;
use raidprotect_util::shutdown::ShutdownSubscriber;
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
    /// In-memory cache
    cache: Arc<InMemoryCache>,
    /// Http client
    http: Arc<HttpClient>,
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

        let cache = Arc::new(InMemoryCache::new(current_user.id));

        let intents = Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_MESSAGES;

        let (cluster, events) = Cluster::builder(token.to_string(), intents)
            .http_client(http.clone())
            .shard_scheme(ShardScheme::Auto)
            .presence(presence())
            .build()
            .await?;

        info!("Started cluster with {} shards", cluster.shards().len());

        Ok(Self {
            cluster: Arc::new(cluster),
            events,
            cache,
            http,
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
                event.process(&self.cache);
            });
        }
    }

    /// Get the cluster [`InMemoryCache`].
    pub fn cache(&self) -> Arc<InMemoryCache> {
        self.cache.clone()
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> Arc<HttpClient> {
        self.http.clone()
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
#[derive(Debug)]
pub enum ClusterError {
    /// HTTP request failed
    Http(HttpError),
    /// Response body deserialization error
    Deserialize(DeserializeBodyError),
    /// Failed to start cluster
    Start(ClusterStartError),
}

impl Error for ClusterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClusterError::Http(e) => Some(e),
            ClusterError::Deserialize(e) => Some(e),
            ClusterError::Start(e) => Some(e),
        }
    }
}

impl Display for ClusterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClusterError::Http(e) => write!(f, "http error: {e}"),
            ClusterError::Deserialize(e) => write!(f, "deserialize error: {e}"),
            ClusterError::Start(e) => write!(f, "failed to start cluster: {e}"),
        }
    }
}

impl From<HttpError> for ClusterError {
    fn from(error: HttpError) -> Self {
        ClusterError::Http(error)
    }
}
impl From<DeserializeBodyError> for ClusterError {
    fn from(error: DeserializeBodyError) -> Self {
        ClusterError::Deserialize(error)
    }
}

impl From<ClusterStartError> for ClusterError {
    fn from(error: ClusterStartError) -> Self {
        ClusterError::Start(error)
    }
}
