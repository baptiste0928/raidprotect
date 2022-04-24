use raidprotect_util::serde::IdAsU64;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

/// Cached model of a [`Channel`].
///
/// Only text channels and threads are cached as the bot
/// does not interact with voice channels.
///
/// [`Channel`]: twilight_model::channel::Channel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CachedChannel {
    /// Text channel.
    Text(CachedTextChannel),
    /// Category channel.
    Category(CachedCategoryChannel),
    /// Public or private thread.
    Thread(CachedThread),
}

impl CachedChannel {
    /// Get the [`Id`] of the channel.
    pub fn id(&self) -> Id<ChannelMarker> {
        match self {
            CachedChannel::Text(channel) => channel.id,
            CachedChannel::Category(channel) => channel.id,
            CachedChannel::Thread(channel) => channel.id,
        }
    }

    /// Get the [`Id`] of the channel guild.
    pub fn guild_id(&self) -> Id<GuildMarker> {
        match self {
            CachedChannel::Text(channel) => channel.guild_id,
            CachedChannel::Category(channel) => channel.guild_id,
            CachedChannel::Thread(channel) => channel.guild_id,
        }
    }

    /// Whether a [`ChannelType`] can be cached with this model.
    pub fn is_cached(kind: ChannelType) -> bool {
        matches!(
            kind,
            ChannelType::GuildText
                | ChannelType::GuildCategory
                | ChannelType::GuildNews
                | ChannelType::GuildPublicThread
                | ChannelType::GuildPrivateThread
                | ChannelType::GuildNewsThread
        )
    }
}

/// Cached model of a text [`Channel`].
///
/// [`Channel`]: twilight_model::channel::Channel
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedTextChannel {
    /// Id of the channel.
    #[serde_as(as = "IdAsU64")]
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the channel belongs.
    #[serde_as(as = "IdAsU64")]
    pub guild_id: Id<GuildMarker>,
    /// Name of the channel.
    pub name: String,
    /// If the channel is in a category, the category id.
    #[serde_as(as = "Option<IdAsU64>")]
    pub parent_id: Option<Id<ChannelMarker>>,
    /// Sorting position of the channel.
    pub position: i64,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u64>,
}

impl From<CachedTextChannel> for CachedChannel {
    fn from(channel: CachedTextChannel) -> Self {
        CachedChannel::Text(channel)
    }
}

/// Cached model of a category [`Channel`].
///
/// [`Channel`]: twilight_model::channel::Channel
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedCategoryChannel {
    /// Id of the category.
    #[serde_as(as = "IdAsU64")]
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the category belongs.
    #[serde_as(as = "IdAsU64")]
    pub guild_id: Id<GuildMarker>,
    /// Name of the category.
    pub name: String,
    /// Sorting position of the category.
    pub position: i64,
    /// Permission overwrites of the category.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

impl From<CachedCategoryChannel> for CachedChannel {
    fn from(channel: CachedCategoryChannel) -> Self {
        CachedChannel::Category(channel)
    }
}

/// Cached model of a public or private thread.
///
/// The bot does not distinguish between private and public threads during processing.
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedThread {
    /// Id of the thread.
    #[serde_as(as = "IdAsU64")]
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the thread belongs.
    #[serde_as(as = "IdAsU64")]
    pub guild_id: Id<GuildMarker>,
    /// Name of the thread.
    pub name: String,
    /// Whether the thread is private.
    pub private: bool,
    /// Parent channel of the thread.
    ///
    /// This field can be [`None`] if the parent channel has been
    /// deleted.
    #[serde_as(as = "Option<IdAsU64>")]
    pub parent_id: Option<Id<ChannelMarker>>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u64>,
}

impl From<CachedThread> for CachedChannel {
    fn from(thread: CachedThread) -> Self {
        CachedChannel::Thread(thread)
    }
}
