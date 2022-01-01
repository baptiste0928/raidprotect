use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use twilight_model::{id::{GuildId, UserId, RoleId, ChannelId}, datetime::Timestamp};

use super::{role::PartialRole, channel::PartialChannel};

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
    pub roles: HashMap<RoleId, PartialRole>,
    /// List of channels of the guild.
    pub channels: HashMap<ChannelId, PartialChannel>,
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
