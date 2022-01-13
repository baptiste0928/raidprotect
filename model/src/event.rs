//! Event models.
//!
//! This module contains models representing Discord events received by theg
//! gateway. They match [`twilight_model::gateway::event`] models with additional
//! fields for cached resources.

use serde::{Deserialize, Serialize};
use twilight_model::{
    guild::{Guild, PartialGuild},
    id::GuildId,
};

use crate::cache::CachedGuild;

/// Event received from Discord.
///
/// This type contain all events that can be received from Discord and handled
/// by the bot. Only event types that are processed are included.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    GuildJoin(GuildJoin),
    GuildUpdate(GuildUpdate),
    GuildLeave(GuildLeave),
}

/// Guild create event.
///
/// This event is only emitted after the shard is ready to avoid sending events
/// while filling the cache.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildJoin {
    pub guild: Guild,
}

/// Guild update event.
///
/// The event is not emitted if the guild was not previously cached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildUpdate {
    pub cached: CachedGuild,
    pub guild: PartialGuild,
}

/// Guild delete event.
///
/// This event is not emitted if the guild was deleted because it becomes
/// unavailable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildLeave {
    pub cached: Option<CachedGuild>,
    pub id: GuildId,
}
