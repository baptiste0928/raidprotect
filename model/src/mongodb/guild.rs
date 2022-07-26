//! Models for the `guilds` collection.

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, with_prefix};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker},
    Id,
};

use crate::serde::IdAsI64;

/// Guild data and configuration.
///
/// This struct correspond to documents stored in the `guilds` collection.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
    /// Name of the MongoDB collection.
    pub const COLLECTION: &'static str = "guilds";

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
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Config {
    /// The channel where RaidProtect send logs messages.
    ///
    /// The configuration validation will fail if no logs chan is set,
    /// but this field may be [`None`] when the initial configuration
    /// has not yet be done.
    #[serde_as(as = "Option<IdAsI64>")]
    pub logs_chan: Option<Id<ChannelMarker>>,
    /// The moderator roles, allowed to access to guild modlogs.
    #[serde_as(as = "Vec<IdAsI64>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub moderator_roles: Vec<Id<RoleMarker>>,
    /// Whether sanction commands requires a reason or not.
    ///
    /// If set to `true`, moderators must specify a reason with each sanction.
    pub enforce_reason: bool,
    /// Whether the moderator who has performed a sanction is hidden for the sanctioned user.
    ///
    /// This is enabled by default.
    pub anonymize_moderator: bool,
    #[serde(flatten, with = "prefix_captcha")]
    pub captcha: Captcha,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logs_chan: None,
            moderator_roles: Vec::new(),
            enforce_reason: false,
            anonymize_moderator: true,
            captcha: Captcha::default(),
        }
    }
}

/// Configuration for the captcha module.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Captcha {
    /// Whether the captcha is enabled.
    pub enabled: bool,
    /// Channel used to send the captcha message.
    ///
    /// This is used to disable the captcha if the channel is deleted.
    #[serde_as(as = "Option<IdAsI64>")]
    pub verification_channel: Option<Id<ChannelMarker>>,
    /// The captcha message id.
    ///
    /// This is used to recreate the captcha message if it is deleted.
    #[serde_as(as = "Option<IdAsI64>")]
    pub verification_message: Option<Id<MessageMarker>>,
    /// Role given to users that haven't completed the captcha.
    #[serde_as(as = "Option<IdAsI64>")]
    pub unverified_role: Option<Id<RoleMarker>>,
    /// Roles given to users after completing the captcha.
    #[serde_as(as = "Vec<IdAsI64>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verified_roles: Vec<Id<RoleMarker>>,
    /// The captcha logs channel.
    ///
    /// If set, the captcha will send detailed logs to this channel.
    #[serde_as(as = "Option<IdAsI64>")]
    pub logs_channel: Option<Id<ChannelMarker>>,
}

with_prefix!(prefix_captcha "captcha_");
