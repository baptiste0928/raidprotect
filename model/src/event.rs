//! Event models.
//!
//! This module contains models representing Discord events received by theg
//! gateway. They match [`twilight_model::gateway::event`] models with additional
//! fields for cached resources.
//!
//! The gateway only send events required by consumers to avoid using unneeded
//! resources.

use serde::{Deserialize, Serialize};
use twilight_model::{channel::Channel, guild::Guild, id::GuildId};

/// Event received from Discord.
///
/// This type contain all events that can be received from Discord and handled
/// by the bot. Only event types that are processed are included.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    GuildJoin(GuildJoin),
    GuildLeave(GuildLeave),
    ChannelCreate(ChannelCreate),
    ChannelUpdate(ChannelUpdate),
    ChannelDelete(ChannelDelete),
}

/// Guild join event.
///
/// This event is only emitted after the shard is ready.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildJoin {
    /// The joined guild.
    pub guild: Guild,
}

/// Guild leave event.
///
/// This event is not emitted when the guild became unavailable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GuildLeave {
    /// ID of the leaved guild.
    pub id: GuildId,
}

/// Channel create event.
///
/// This event is only emitted after the shard is ready.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelCreate {
    /// The created channel.
    pub channel: Channel,
}

/// Channel update event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelUpdate {
    /// The updated channel.
    pub channel: Channel,
}

/// Channel delete event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelDelete {
    /// The deleted channel.
    pub channel: Channel,
}
