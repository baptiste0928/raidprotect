use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use raidprotect_model::cache::UpdateCache;
use tracing::{debug, error, trace};
use twilight_model::{
    application::interaction::Interaction,
    gateway::{event::Event as GatewayEvent, payload::incoming},
};

use super::message::ALLOWED_MESSAGES_TYPES;
use crate::cluster::ClusterState;

/// Process incoming events.
#[async_trait]
pub trait ProcessEvent: Sized {
    /// Process incoming event.
    async fn process(self, state: Arc<ClusterState>);
}

macro_rules! process_events {
    ($self:ident, $state:ident => $( $event:path ),+ ) => {
        match $self {
            $(
                $event(event) => event.process($state).await,
            )+
            event => trace!(kind = event.kind().name(), "unprocessed event type"),
        }
    };
}

async fn process_cache_event<E: UpdateCache + Debug>(event: E, state: &ClusterState) {
    if let Err(error) = event.update(state.redis(), state.current_user()).await {
        error!(error = ?error, kind = E::NAME, "failed to update cache");
        debug!(event = ?event);
    }
}

macro_rules! process_cache_events {
    ( $( $event:ident ),+ ) => {
        $(
            #[async_trait]
            impl ProcessEvent for incoming::$event {
                async fn process(self, state: Arc<ClusterState>) {
                    process_cache_event(self, &state).await;
                }
            }
        )+
    };
}

#[async_trait]
impl ProcessEvent for GatewayEvent {
    async fn process(self, state: Arc<ClusterState>) {
        use GatewayEvent::*;

        // `self` is renamed `__self` in async_trait macro expansion
        process_events! { __self, state =>
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

#[async_trait]
impl ProcessEvent for incoming::InteractionCreate {
    async fn process(self, state: Arc<ClusterState>) {
        match self.0 {
            Interaction::ApplicationCommand(command) => {
                crate::interaction::handle_command(*command, state).await;
            }
            Interaction::MessageComponent(component) => {
                crate::interaction::handle_component(*component, state).await;
            }
            Interaction::ModalSubmit(modal) => {
                crate::interaction::handle_modal(*modal, state).await;
            }
            _ => {
                trace!("unprocessed interaction type");
            }
        };
    }
}

#[async_trait]
impl ProcessEvent for incoming::MessageCreate {
    async fn process(self, state: Arc<ClusterState>) {
        if self.guild_id.is_some() && ALLOWED_MESSAGES_TYPES.contains(&self.kind) {
            super::message::handle_message(self.0, state).await;
        }
    }
}
