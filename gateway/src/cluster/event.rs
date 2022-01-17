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

macro_rules! process_events {
    ($self:ident, $cache:ident, $broadcast:ident => $( $event:path ),+ ) => {
        match $self {
            $(
                $event(event) => event.process($cache, $broadcast),
            )+
            event => trace!(kind = event.kind().name(), "unprocessed event type"),
        }
    };
}

impl ProcessEvent for GatewayEvent {
    fn process(self, cache: &InMemoryCache, broadcast: &Sender<Event>) {
        use GatewayEvent::*;

        process_events! { self, cache, broadcast =>
            GuildCreate,
            GuildDelete,
            GuildUpdate,
            ChannelCreate,
            ChannelDelete,
            ChannelUpdate,
            ThreadCreate,
            ThreadDelete,
            ThreadUpdate,
            RoleCreate,
            RoleDelete,
            MemberAdd,
            MemberUpdate
        }
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
