use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker},
    Id,
};

use super::serde_helpers::{id_as_i64, option_id_as_i64};

/// Name of the `guilds` collection
pub const GUILDS_COLLECTION: &str = "guilds";

/// Guild data and configuration.
///
/// This struct correspond to documents stored in the `guilds` collection.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
    /// Discord guild id.
    #[serde(rename = "_id", with = "id_as_i64")]
    pub id: Id<GuildMarker>,
}

/// A guild configuration.
///
/// Stored in the `guilds` collection within a [`Guild`].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// The channel where RaidProtect send logs messages.
    ///
    /// The configuration validation will fail if no logs chan is set,
    /// but this field may be [`None`] when the initial configuration
    /// has not yet be done.
    #[serde(with = "option_id_as_i64")]
    pub logs_chan: Option<Id<ChannelMarker>>,
    /// The moderator roles, allowed to access to guild modlogs
    /// and to perform some sanction (mute and warn).
    pub moderator_roles: Vec<Id<RoleMarker>>,
    /// Whether sanction commands requires a reason or not.
    pub enforce_reason: bool,
    /// Whether mute command allow infinite mute (without explicit duration set).
    pub infinite_mute: bool,
    /// Whether the moderator who has performed a sanction is hidden for the sanctioned user.
    pub anonymize_moderator: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logs_chan: None,
            moderator_roles: Vec::new(),
            enforce_reason: false,
            infinite_mute: false,
            anonymize_moderator: true,
        }
    }
}
