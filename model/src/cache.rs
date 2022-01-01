//! Cache models.
//!
//! This module contains models used by the cache. These models
//! are based on [`twilight_model`] models but without unnecessary
//! fields to decrease memory usage.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite,
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
    pub position: i64,
    /// Permissions of the role.
    pub permissions: Permissions,
}

/// Cached model of a [`GuildChannel`].
///
/// Only text channels and threads are cached as the bot
/// does not interact with voice channels.
///
/// [`GuildChannel`]: twilight_model::channel::GuildChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CachedChannel {
    /// Text channel.
    Text(CachedTextChannel),
    /// Category channel.
    Category(CachedCategoryChannel),
    /// Public or private thread.
    Thread(CachedThread),
}

/// Cached model of a [`TextChannel`].
///
/// [`TextChannel`]: twilight_model::channel::TextChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedTextChannel {
    /// Id of the channel.
    pub id: ChannelId,
    /// Id of the guild to which the channel belongs.
    pub guild_id: GuildId,
    /// Name of the channel.
    pub name: String,
    /// If the channel is in a category, the category id.
    pub parent_id: Option<ChannelId>,
    /// Sorting position of the channel.
    pub position: i64,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u64>,
}

/// Cached model of a [`CategoryChannel`].
///
/// [`CategoryChannel`]: twilight_model::channel::CategoryChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedCategoryChannel {
    /// Id of the category.
    pub id: ChannelId,
    /// Id of the guild to which the category belongs.
    pub guild_id: GuildId,
    /// Name of the category.
    pub name: String,
    /// Sorting position of the category.
    pub position: i64,
    /// Permission overwrites of the category.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

/// Cached model of a [`PublicThread`] or [`PrivateThread`].
///
/// The bot does not distinguish between private and public threads during processing.
///
/// [`PublicThread`]: twilight_model::channel::thread::PublicThread
/// [`PrivateThread`]: twilight_model::channel::thread::PrivateThread
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedThread {
    /// Id of the thread.
    pub id: ChannelId,
    /// Id of the guild to which the thread belongs.
    pub guild_id: GuildId,
    /// Name of the thread.
    pub name: String,
    /// Whether the thread is private.
    pub private: bool,
    /// Parent channel of the thread.
    ///
    /// This field can be [`None`] if the parent channel has been
    /// deleted.
    pub parent_id: Option<ChannelId>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u64>,
}

/// Partial model of a [`GuildChannel`].
///
/// This type is used in [`CachedGuild`] and only contain fields
/// required for permissions calculation. Only text channels and threads
/// are cached as the bot does not interact with voice channels.
///
/// [`GuildChannel`]: twilight_model::channel::GuildChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PartialChannel {
    /// Partial text channel.
    Text(PartialTextChannel),
    /// Partial category channel.
    Category(PartialCategoryChannel),
    /// Partial thread.
    Thread(PartialThread),
}

/// Partial model of a [`TextChannel`].
///
/// [`TextChannel`]: twilight_model::channel::TextChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialTextChannel {
    /// If the channel is in a category, the category id.
    pub parent_id: Option<ChannelId>,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

/// Partial model of a [`CategoryChannel`].
///
/// [`CategoryChannel`]: twilight_model::channel::CategoryChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialCategoryChannel {
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

/// Cached model of a [`PublicThread`] or [`PrivateThread`].
///
/// [`PublicThread`]: twilight_model::channel::thread::PublicThread
/// [`PrivateThread`]: twilight_model::channel::thread::PrivateThread
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialThread {
    /// Parent channel of the thread.
    pub parent_id: Option<ChannelId>,
}
