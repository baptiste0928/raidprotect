//! Discord shards cluster.

use std::sync::Arc;

use futures::StreamExt;
use raidprotect_util::shutdown::ShutdownSubscriber;
use tokio::sync::broadcast;
use tracing::{debug, instrument};
use twilight_gateway::{
    cluster::{ClusterStartError, Events, ShardScheme},
    Cluster, Event, Intents,
};
use twilight_model::gateway::{
    payload::outgoing::update_presence::UpdatePresencePayload,
    presence::{ActivityType, MinimalActivity, Status},
};

/// Wrapper around a twilight [`Cluster`].
#[derive(Debug)]
pub struct ShardCluster {
    /// Broadcast sender channel
    pub broadcast: broadcast::Sender<Event>,
    /// Inner shard cluster managed by twilight
    cluster: Arc<Cluster>,
    /// Events stream
    events: Events,
}

impl ShardCluster {
    /// Initialize a new shards cluster without starting it.
    pub async fn new(token: &str) -> Result<Self, ClusterStartError> {
        let intents = Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_MESSAGES;

        let (cluster, events) = Cluster::builder(token, intents)
            .shard_scheme(ShardScheme::Auto)
            .presence(presence())
            .build()
            .await?;

        let (sender, _) = broadcast::channel(16);

        Ok(Self {
            broadcast: sender,
            cluster: Arc::new(cluster),
            events,
        })
    }

    /// Start the cluster and handle incoming events.
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
            debug!(event = ?event, "received event")
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
