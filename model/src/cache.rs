//! Cache models.
//!
//! This module contains models used by the cache. These models
//! are based on [`twilight_model`] models but without unnecessary
//! fields to decrease memory usage.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    datetime::Timestamp,
    guild::Permissions,
    id::{ChannelId, GuildId, RoleId, UserId},
};

/// Cached model of a [`Guild`].
///
/// [`Guild`]: twilight_model::guild::Guild
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedGuild {
    /// Id of the guild.
    id: GuildId,
    /// Name of the guild
    name: String,
    /// Hash of the guild icon.
    icon: Option<String>,
    /// Id of the guild's owner.
    owner_id: UserId,
    /// Information about the bot member in the guild.
    ///
    /// If this field is [`None`], the information has not been
    /// properly received and all permission calculations should fail.
    current_member: Option<CurrentMember>,
    /// List of roles of the guild.
    roles: HashMap<RoleId, PartialRole>,
    /// List of channels of the guild.
    channels: HashMap<ChannelId, PartialChannel>,
}

/// Information about the bot [`Member`] in a guild.
///
/// [`Member`]: twilight_model::guild::member::Member
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CurrentMember {
    /// Id of the bot current member.
    id: UserId,
    /// When the bot can resume communication in a guild again.
    ///
    /// Checking if this value is [`Some`] is not enough, we should
    /// also check that the given timestamp is not in the past.
    communication_disabled_until: Option<Timestamp>,
    /// Roles of the bot.
    roles: Vec<RoleId>,
}

/// Partial model of a [`Role`].
///
/// This type is used in [`CachedGuild`] and only contain fields
/// required for permissions calculation.
///
/// [`Role`]: twilight_model::guild::Role
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialRole {
    /// Position of the role.
    ///
    /// The position *should* be positive but can be negative
    /// in some cases. Only the ordering is important for
    /// permission calculations.
    position: i64,
    /// Permissions of the role.
    permissions: Permissions,
}

/// Partial model of a guild [`Channel`].
///
/// This type is used in [`CachedGuild`] and only contains fields
/// required for permissions calculation.
///
/// [`Channel`]: twilight_model::channel::Channel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialChannel {
    /// Type of channel (text, category, voice, ...)
    kind: ChannelType,
    permission_overwrites: Vec<PermissionOverwrite>,
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
    id: RoleId,
    /// Name of the role.
    name: String,
    /// Color of the role.
    color: u32,
    /// Icon image hash.
    icon: Option<String>,
    /// Icon unicode emoji.
    ///
    /// This field is set if the role has an icon which is
    /// not a custom image but an existing unicode emoji.
    unicode_emoji: Option<String>,
    /// Position of the role.
    position: i64,
    /// Permissions of the role.
    permissions: Permissions,
    /// Whether the role is managed.
    ///
    /// Managed roles include bot, integration or boost roles.
    managed: bool,
}
