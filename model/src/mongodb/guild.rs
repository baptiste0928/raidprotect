//! Models for the `guilds` collection.

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, with_prefix};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker},
    Id,
};

use crate::serde::IdAsI64;

/// Guild configuration.
///
/// This struct correspond to documents stored in the `guilds` collection.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Guild {
    /// Discord guild id.
    #[serde_as(as = "IdAsI64")]
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
    /// The channel where RaidProtect send logs messages.
    ///
    /// The configuration validation will fail if no logs chan is set,
    /// but this field may be [`None`] when the initial configuration
    /// has not yet be done.
    #[serde_as(as = "Option<IdAsI64>")]
    #[serde(default)]
    pub logs_chan: Option<Id<ChannelMarker>>,
    /// Lang used for the global guild messages.
    #[serde(default = "Guild::default_lang")]
    pub lang: String,
    /// The moderation module configuration.
    #[serde(default, flatten, with = "prefix_moderation")]
    pub moderation: Moderation,
    /// The captcha module configuration.
    #[serde(default, flatten, with = "prefix_captcha")]
    pub captcha: Captcha,
}

impl Guild {
    /// Name of the MongoDB collection.
    pub const COLLECTION: &'static str = "guilds";

    /// Initialize a new [`Guild`] with default configuration.
    pub fn new(id: Id<GuildMarker>) -> Self {
        Self {
            id,
            logs_chan: None,
            lang: Self::default_lang(),
            moderation: Moderation::default(),
            captcha: Captcha::default(),
        }
    }

    fn default_lang() -> String {
        "fr".to_string() // TODO: change default lang to english
    }
}

/// Configuration for the moderation module.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct Moderation {
    /// The moderator roles, allowed to access to guild modlogs.
    #[serde_as(as = "Vec<IdAsI64>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<Id<RoleMarker>>,
    /// Whether sanction commands requires a reason or not.
    ///
    /// If set to `true`, moderators must specify a reason with each sanction.
    pub enforce_reason: bool,
    /// Whether the moderator who has performed a sanction is hidden for the sanctioned user.
    ///
    /// This is enabled by default.
    pub anonymize: bool,
}

impl Default for Moderation {
    fn default() -> Self {
        Self {
            roles: Vec::new(),
            enforce_reason: false,
            anonymize: true,
        }
    }
}

with_prefix!(prefix_moderation "moderation_");

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
    pub channel: Option<Id<ChannelMarker>>,
    /// The captcha message id.
    ///
    /// This is used to recreate the captcha message if it is deleted.
    #[serde_as(as = "Option<IdAsI64>")]
    pub message: Option<Id<MessageMarker>>,
    /// Role given to users that haven't completed the captcha.
    #[serde_as(as = "Option<IdAsI64>")]
    pub role: Option<Id<RoleMarker>>,
    /// Roles given to users after completing the captcha.
    #[serde_as(as = "Vec<IdAsI64>")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verified_roles: Vec<Id<RoleMarker>>,
    /// The captcha logs channel.
    ///
    /// If set, the captcha will send detailed logs to this channel.
    #[serde_as(as = "Option<IdAsI64>")]
    pub logs: Option<Id<ChannelMarker>>,
}

with_prefix!(prefix_captcha "captcha_");
