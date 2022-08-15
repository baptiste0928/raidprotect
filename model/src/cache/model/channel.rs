use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

use crate::{cache::RedisModel, serde::IdAsU64};

/// Cached model of a [`Channel`].
///
/// [`Channel`]: twilight_model::channel::Channel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum CachedChannel {
    /// Text channel.
    Text(CachedTextChannel),
    /// Voice channel.
    Voice(CachedVoiceChannel),
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
            CachedChannel::Voice(channel) => channel.id,
            CachedChannel::Category(channel) => channel.id,
            CachedChannel::Thread(channel) => channel.id,
        }
    }

    /// Get the [`Id`] of the channel.
    pub fn guild_id(&self) -> Id<GuildMarker> {
        match self {
            CachedChannel::Text(channel) => channel.guild_id,
            CachedChannel::Voice(channel) => channel.guild_id,
            CachedChannel::Category(channel) => channel.guild_id,
            CachedChannel::Thread(channel) => channel.guild_id,
        }
    }

    /// Get the [`ChannelType`] of the channel.
    ///
    /// The exact type of the channel is not currently stored in the cache, so
    /// this function will return a type corresponding to the enum variant.
    pub fn kind(&self) -> ChannelType {
        match self {
            CachedChannel::Text(_) => ChannelType::GuildText,
            CachedChannel::Voice(_) => ChannelType::GuildVoice,
            CachedChannel::Category(_) => ChannelType::GuildCategory,
            CachedChannel::Thread(_) => ChannelType::GuildPublicThread,
        }
    }

    /// Get the [`PermissionOverwrite`]s of the channel.
    ///
    /// Note that no permissions are returned for thread channels.
    pub fn permissions(&self) -> &[PermissionOverwrite] {
        match self {
            CachedChannel::Text(channel) => &channel.permission_overwrites,
            CachedChannel::Voice(channel) => &channel.permission_overwrites,
            CachedChannel::Category(channel) => &channel.permission_overwrites,
            CachedChannel::Thread(_) => &[],
        }
    }

    /// Whether a [`ChannelType`] can be cached with this model.
    ///
    /// KEEP IN SYNC with `cache_guild_channel` in `model/src/cache/process/resource.rs`.
    pub fn is_cached(kind: ChannelType) -> bool {
        match kind {
            ChannelType::GuildText
            | ChannelType::GuildVoice
            | ChannelType::GuildStageVoice
            | ChannelType::GuildCategory
            | ChannelType::GuildNews
            | ChannelType::GuildPublicThread
            | ChannelType::GuildPrivateThread
            | ChannelType::GuildNewsThread => true,
            ChannelType::Private
            | ChannelType::Group
            | ChannelType::GuildDirectory
            | ChannelType::GuildForum
            | ChannelType::Unknown(_) => false,
        }
    }
}

impl RedisModel for CachedChannel {
    type Id = Id<ChannelMarker>;

    fn key(&self) -> String {
        Self::key_from(&self.id())
    }

    fn key_from(id: &Self::Id) -> String {
        format!("c:channel:{id}")
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
    pub position: i32,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u16>,
}

impl From<CachedTextChannel> for CachedChannel {
    fn from(channel: CachedTextChannel) -> Self {
        CachedChannel::Text(channel)
    }
}

/// Cached model of a voice [`Channel`].
///
/// [`Channel`]: twilight_model::channel::Channel
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedVoiceChannel {
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
    pub position: i32,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

impl From<CachedVoiceChannel> for CachedChannel {
    fn from(channel: CachedVoiceChannel) -> Self {
        CachedChannel::Voice(channel)
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
    pub position: i32,
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
    #[serde_as(as = "IdAsU64")]
    pub parent_id: Id<ChannelMarker>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u16>,
}

impl From<CachedThread> for CachedChannel {
    fn from(thread: CachedThread) -> Self {
        CachedChannel::Thread(thread)
    }
}
