use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker},
    Id,
};

use super::serde_helpers::IdAsI64;

/// Name of the `guilds` collection
pub const GUILDS_COLLECTION: &str = "guilds";

/// Guild data and configuration.
///
/// This struct correspond to documents stored in the `guilds` collection.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Guild {
    /// Discord guild id.
    #[serde_as(as = "IdAsI64")]
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
    /// The guild configuration.
    #[serde(default)]
    pub config: Config,
}

impl Guild {
    /// Initialize a new [`Guild`] with default configuration.
    pub fn new(id: Id<GuildMarker>) -> Self {
        Self {
            id,
            config: Config::default(),
        }
    }
}

/// A guild configuration.
///
/// Stored in the `guilds` collection within a [`Guild`].
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    /// The channel where RaidProtect send logs messages.
    ///
    /// The configuration validation will fail if no logs chan is set,
    /// but this field may be [`None`] when the initial configuration
    /// has not yet be done.
    #[serde_as(as = "Option<IdAsI64>")]
    pub logs_chan: Option<Id<ChannelMarker>>,
    /// The moderator roles, allowed to access to guild modlogs
    /// and to perform some sanction (mute and warn).
    pub moderator_roles: Vec<Id<RoleMarker>>,
    /// Whether sanction commands requires a reason or not.
    ///
    /// If set to `true`, moderators must specify a reason with each sanction.
    pub enforce_reason: bool,
    /// Whether the moderator who has performed a sanction is hidden for the sanctioned user.
    pub anonymize_moderator: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logs_chan: None,
            moderator_roles: Vec::new(),
            enforce_reason: false,
            anonymize_moderator: true,
        }
    }
}
