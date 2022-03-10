//! Event context.
//!
//! These structs represent the context of an event, with additional information
//! retrieved from cache or database.

use std::sync::Arc;

use raidprotect_cache::model::CachedGuild;
use thiserror::Error;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::cluster::ClusterState;

/// Context of a received event.
///
/// If the event occurred in a guild, a [`CachedGuild`] will be included.
#[derive(Debug, Clone)]
pub struct EventContext {
    /// Shared cluster state.
    pub state: Arc<ClusterState>,
    /// If event occurred in a guild, the cached guild.
    pub guild: Option<CachedGuild>,
}

impl EventContext {
    pub(crate) fn new(
        state: Arc<ClusterState>,
        guild_id: Option<Id<GuildMarker>>,
    ) -> Result<Self, EventContextError> {
        let guild = match guild_id {
            Some(id) => Some(
                state
                    .cache()
                    .guild(id)
                    .ok_or(EventContextError::GuildNotFound)?
                    .clone(),
            ),
            None => None,
        };

        Ok(Self { state, guild })
    }
}

/// Error occured when intializing [`EventContext`].
#[derive(Debug, Error)]
pub enum EventContextError {
    #[error("guild not found in cache")]
    GuildNotFound,
}
