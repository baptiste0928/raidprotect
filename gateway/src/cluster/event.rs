use async_trait::async_trait;
use raidprotect_model::event::Event;
use tokio::sync::broadcast::Sender;
use tracing::debug;
use twilight_model::gateway::event::Event as GatewayEvent;

use crate::cache::InMemoryCache;

/// Process incoming events.
#[async_trait]
pub trait ProcessEvent: Sized {
    /// Process incoming event.
    async fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>);
}

#[async_trait]
impl ProcessEvent for GatewayEvent {
    async fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>) {
        match self {
            event => debug!(event = ?event, "received unprocessed event"),
        }
    }
}
