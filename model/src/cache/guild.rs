use serde::{Deserialize, Serialize};
use twilight_model::{
    datetime::Timestamp,
    guild::Permissions,
    id::{ChannelId, GuildId, RoleId, UserId},
};

use super::partial::{IntoPartial, PartialRole};

/// Cached model of a [`Guild`].
///
/// [`Guild`]: twilight_model::guild::Guild
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedGuild {
    /// Id of the guild.
    pub id: GuildId,
    /// Name of the guild
    pub name: String,
    /// Hash of the guild icon.
    pub icon: Option<String>,
    /// Id of the guild's owner.
    pub owner_id: UserId,
    /// Information about the bot member in the guild.
    ///
    /// If this field is [`None`], the information has not been
    /// properly received and all permission calculations should fail.
    pub current_member: Option<CurrentMember>,
    /// List of roles of the guild.
    pub roles: Vec<RoleId>,
    /// List of channels of the guild.
    pub channels: Vec<ChannelId>,
}

/// Information about the bot [`Member`] in a guild.
///
/// [`Member`]: twilight_model::guild::member::Member
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CurrentMember {
    /// Id of the bot current member.
    pub id: UserId,
    /// When the bot can resume communication in a guild again.
    ///
    /// Checking if this value is [`Some`] is not enough, we should
    /// also check that the given timestamp is not in the past.
    pub communication_disabled_until: Option<Timestamp>,
    /// Roles of the bot.
    pub roles: Vec<RoleId>,
}

/// Cached model of a [`Role`].
///
/// This model is not cached within guilds to limit
/// data to send when requesting a [`CachedGuild`].
///
/// [`Role`]: twilight_model::guild::Role
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedRole {
    /// Id of the role.
    pub id: RoleId,
    /// Id of the guild to which the role belongs.
    pub guild_id: GuildId,
    /// Name of the role.
    pub name: String,
    /// Color of the role.
    pub color: u32,
    /// Icon image hash.
    pub icon: Option<String>,
    /// Icon unicode emoji.
    ///
    /// This field is set if the role has an icon which is
    /// not a custom image but an existing unicode emoji.
    pub unicode_emoji: Option<String>,
    /// Position of the role.
    pub position: i64,
    /// Permissions of the role.
    pub permissions: Permissions,
    /// Whether the role is managed.
    ///
    /// Managed roles include bot, integration or boost roles.
    pub managed: bool,
}

impl IntoPartial for CachedRole {
    type Partial = PartialRole;

    fn into_partial(&self) -> Self::Partial {
        PartialRole {
            position: self.position,
            permissions: self.permissions,
        }
    }
}
