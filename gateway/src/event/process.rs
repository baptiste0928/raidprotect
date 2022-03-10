use std::sync::Arc;

use tracing::{error, trace};
use twilight_model::{
    application::interaction::Interaction,
    gateway::{event::Event as GatewayEvent, payload::incoming},
};

use crate::cluster::ClusterState;

use super::context::EventContext;

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
    fn process(self, state: Arc<ClusterState>) {
        match self.0 {
            Interaction::ApplicationCommand(command) => {
                let _context = match EventContext::new(state, command.guild_id) {
                    Ok(context) => context,
                    Err(error) => {
                        error!(error = %error, "failed to initialize event context");
                        return;
                    }
                };

                todo!("spawn handler")
            }
            _ => todo!(),
        }
    }
}
