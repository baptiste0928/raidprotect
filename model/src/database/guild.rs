//! Models for the `guilds` collection.

use anyhow::Context;
use mongodb::{
    bson::{doc, to_document},
    options,
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker},
    Id,
};

use super::DbClient;
use crate::serde::IdAsI64;

/// Guild configuration.
///
/// This type represent a guild configuration stored in the `guilds` collection
/// of the database.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GuildConfig {
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
    #[serde(default = "default_lang")]
    pub lang: String,
    /// The moderation module configuration.
    #[serde(default)]
    pub moderation: ModerationConfig,
    /// The captcha module configuration.
    #[serde(default)]
    pub captcha: CaptchaConfig,
}

fn default_lang() -> String {
    "fr".to_string() // TODO: change default lang to english
}

impl GuildConfig {
    /// Name of the MongoDB collection.
    pub const COLLECTION: &'static str = "guilds";

    /// Initialize a new [`GuildConfig`] with default configuration.
    pub fn new(id: Id<GuildMarker>) -> Self {
        Self {
            id,
            logs_chan: None,
            lang: default_lang(),
            moderation: ModerationConfig::default(),
            captcha: CaptchaConfig::default(),
        }
    }
}

/// Configuration for the moderation module.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct ModerationConfig {
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

impl Default for ModerationConfig {
    fn default() -> Self {
        Self {
            roles: Vec::new(),
            enforce_reason: false,
            anonymize: true,
        }
    }
}

/// Configuration for the captcha module.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct CaptchaConfig {
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

impl CaptchaConfig {
    /// Max length of the `verified_roles` field.
    pub const MAX_VERIFIED_ROLES_LEN: usize = 5;
}

// Implementation of methods to query the database.
impl DbClient {
    /// Get the [`GuildConfig`] for a given guild_id, if it exists.
    pub async fn get_guild(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Option<GuildConfig>, anyhow::Error> {
        let query = GuildQuery { id: guild_id };

        let guild = self
            .db()
            .collection::<GuildConfig>(GuildConfig::COLLECTION)
            .find_one(to_document(&query)?, None)
            .await?;

        Ok(guild)
    }

    /// Get the [`GuildConfig`] for a given guild_id, or create it with default configuration.
    pub async fn get_guild_or_create(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<GuildConfig, anyhow::Error> {
        let query = GuildQuery { id: guild_id };
        let default_guild = GuildConfig::new(guild_id);
        let options = options::FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(options::ReturnDocument::After)
            .build();

        let guild = self
            .db()
            .collection::<GuildConfig>(GuildConfig::COLLECTION)
            .find_one_and_update(
                to_document(&query)?,
                doc! { "$setOnInsert": to_document(&default_guild)? },
                options,
            )
            .await?;

        guild.context("no guild sent by the database")
    }

    /// Update or insert a [`GuildConfig`] in the database.
    pub async fn update_guild(&self, guild: &GuildConfig) -> Result<(), anyhow::Error> {
        let query = GuildQuery { id: guild.id };
        let options = options::ReplaceOptions::builder().upsert(true).build();

        self.db()
            .collection::<GuildConfig>(GuildConfig::COLLECTION)
            .replace_one(to_document(&query)?, guild, options)
            .await?;

        Ok(())
    }
}

/// Query a guild with its guild_id
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct GuildQuery {
    #[serde_as(as = "IdAsI64")]
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
}
