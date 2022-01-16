use match_any::match_any;
use raidprotect_model::event::{Event, InteractionCreate};
use tokio::sync::broadcast::Sender;
use tracing::trace;
use twilight_model::gateway::{
    event::Event as GatewayEvent,
    payload::incoming::{self},
};

use crate::cache::{InMemoryCache, UpdateCache};

/// Process incoming events.
pub trait ProcessEvent: Sized {
    /// Process incoming event.
    fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>);
}

impl ProcessEvent for GatewayEvent {
    fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>) {
        use GatewayEvent::*;

        match_any!(self,
            GuildCreate(event)
            | GuildDelete(event)
            | GuildUpdate(event)
            | ChannelCreate(event)
            | ChannelDelete(event)
            | ChannelUpdate(event)
            | ThreadCreate(event)
            | ThreadDelete(event)
            | ThreadUpdate(event)
            | RoleCreate(event)
            | RoleDelete(event)
            | MemberAdd(event)
            | MemberUpdate(event) => event.process(cache, broadcast),
            event => trace!(kind = event.kind().name(), "unprocessed event type")
        )
    }
}

impl<T> ProcessEvent for T
where
    T: UpdateCache + Send + Sync,
{
    fn process(self, cache: &InMemoryCache, _broadcast: &Sender<Event>) {
        cache.update(&self);
    }
}

impl ProcessEvent for incoming::InteractionCreate {
    fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>) {
        let guild = self.guild_id().map(|id| cache.guild(id)).flatten();

        let _ = broadcast.send(Event::InteractionCreate(InteractionCreate {
            guild,
            interaction: self.0,
        }));
    }
}
