use raidprotect_cache::InMemoryCache;
use tracing::trace;
use twilight_model::gateway::{event::Event as GatewayEvent, payload::incoming};

/// Process incoming events.
pub trait ProcessEvent: Sized {
    /// Process incoming event.
    fn process(self, cache: &InMemoryCache);
}

macro_rules! process_events {
    ($self:ident, $cache:ident => $( $event:path ),+ ) => {
        match $self {
            $(
                $event(event) => event.process($cache),
            )+
            event => trace!(kind = event.kind().name(), "unprocessed event type"),
        }
    };
}

macro_rules! process_cache_events {
    ( $( $event:ident ),+ ) => {
        $(
            impl ProcessEvent for incoming::$event {
                fn process(self, cache: &InMemoryCache) {
                    cache.update(&self);
                }
            }
        )+
    };
}

impl ProcessEvent for GatewayEvent {
    fn process(self, cache: &InMemoryCache) {
        use GatewayEvent::*;

        process_events! { self, cache =>
            GuildCreate,
            GuildDelete,
            UnavailableGuild,
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

// Implementation of events only processed in cache
process_cache_events! {
    GuildCreate,
    GuildDelete,
    UnavailableGuild,
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

impl ProcessEvent for incoming::InteractionCreate {
    fn process(self, cache: &InMemoryCache) {
        let _guild = self.guild_id().map(|id| cache.guild(id)).flatten();

        todo!("handle interactions")
    }
}
