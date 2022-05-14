//! Get the logs channel of a guild.
//!
//! This module exports functions to get the logs channel of a specific guild.
//!
//! In case the channel is not configured for the current guild, a new one will
//! be automatically created. If a channel named `raidprotect-logs` is already
//! present, it will be reused.
//!
//! A simple locking mechanism is used to prevent multiple channels to be created
//! at the same time.

use raidprotect_state::ClusterState;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker},
    Id,
};

/// Get the logs channel of a guild.
///
/// See the [module documentation](super) for more information.
pub async fn guild_logs_channel(
    _guild_id: Id<GuildMarker>,
    _logs_chan: Option<Id<ChannelMarker>>,
    _state: &ClusterState,
) {
    todo!()
}
