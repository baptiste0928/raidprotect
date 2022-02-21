//! Shards cluster implementation.

use std::sync::Arc;

use futures::StreamExt;
use raidprotect_cache::InMemoryCache;
use raidprotect_util::shutdown::ShutdownSubscriber;
use tracing::{info_span, instrument, trace};
use twilight_gateway::{
    cluster::{ClusterStartError, Events, ShardScheme},
    Cluster, Intents,
};
use twilight_http::Client as HttpClient;
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
}

impl ShardCluster {
    /// Initialize a new [`ShardCluster`].
    ///
    /// This method takes an inital [`HttpClient`] and [`InMemoryCache`].
    pub async fn new(
        token: &str,
        http: Arc<HttpClient>,
        cache: Arc<InMemoryCache>,
    ) -> Result<Self, ClusterStartError> {
        let intents = Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_MESSAGES;

        let (cluster, events) = Cluster::builder(token.to_string(), intents)
            .http_client(http)
            .shard_scheme(ShardScheme::Auto)
            .presence(presence())
            .build()
            .await?;

        Ok(Self {
            cluster: Arc::new(cluster),
            events,
            cache,
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
