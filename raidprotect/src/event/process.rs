use std::sync::Arc;

use raidprotect_event::message::ALLOWED_MESSAGES_TYPES;
use raidprotect_model::ClusterState;
use tracing::trace;
use twilight_model::{
    application::interaction::Interaction,
    gateway::{event::Event as GatewayEvent, payload::incoming},
};

/// Process incoming events.
pub trait ProcessEvent: Sized {
    /// Process incoming event.
    fn process(self, state: Arc<ClusterState>);
}

macro_rules! process_events {
    ($self:ident, $state:ident => $( $event:path ),+ ) => {
        match $self {
            $(
                $event(event) => event.process($state),
            )+
            event => trace!(kind = event.kind().name(), "unprocessed event type"),
        }
    };
}

macro_rules! process_cache_events {
    ( $( $event:ident ),+ ) => {
        $(
            impl ProcessEvent for incoming::$event {
                fn process(self, state: Arc<ClusterState>) {
                    state.cache().update(&self);
                }
            }
        )+
    };
}

impl ProcessEvent for GatewayEvent {
    fn process(self, state: Arc<ClusterState>) {
        use GatewayEvent::*;

        process_events! { self, state =>
            GuildCreate,
            GuildDelete,
            UnavailableGuild,
            GuildUpdate,
            ChannelCreate,
            ChannelDelete,
            ChannelUpdate,
            InteractionCreate,
            ThreadCreate,
            ThreadDelete,
            ThreadUpdate,
            RoleCreate,
            RoleDelete,
            MemberAdd,
            MemberUpdate,
            MessageCreate
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
    fn process(self, state: Arc<ClusterState>) {
        match self.0 {
            Interaction::ApplicationCommand(command) => {
                tokio::spawn(raidprotect_interaction::handle_command(*command, state));
            }
            Interaction::MessageComponent(component) => {
                tokio::spawn(raidprotect_interaction::handle_component(*component, state));
            }
            _ => {
                trace!("unprocessed interaction type");
            }
        };
    }
}

impl ProcessEvent for incoming::MessageCreate {
    fn process(self, state: Arc<ClusterState>) {
        if self.guild_id.is_some() && ALLOWED_MESSAGES_TYPES.contains(&self.kind) {
            tokio::spawn(raidprotect_event::message::handle_message(self.0, state));
        }
    }
}
