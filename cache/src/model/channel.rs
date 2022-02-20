use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType},
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};

/// Cached model of a [`GuildChannel`].
///
/// Only text channels and threads are cached as the bot
/// does not interact with voice channels.
///
/// [`GuildChannel`]: twilight_model::channel::GuildChannel
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Cached model of a [`TextChannel`].
///
/// [`TextChannel`]: twilight_model::channel::TextChannel
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedTextChannel {
    /// Id of the channel.
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the channel belongs.
    pub guild_id: Id<GuildMarker>,
    /// Name of the channel.
    pub name: String,
    /// If the channel is in a category, the category id.
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

/// Cached model of a [`CategoryChannel`].
///
/// [`CategoryChannel`]: twilight_model::channel::CategoryChannel
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedCategoryChannel {
    /// Id of the category.
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the category belongs.
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

/// Cached model of a [`PublicThread`] or [`PrivateThread`].
///
/// The bot does not distinguish between private and public threads during processing.
///
/// [`PublicThread`]: twilight_model::channel::thread::PublicThread
/// [`PrivateThread`]: twilight_model::channel::thread::PrivateThread
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedThread {
    /// Id of the thread.
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the thread belongs.
    pub guild_id: Id<GuildMarker>,
    /// Name of the thread.
    pub name: String,
    /// Whether the thread is private.
    pub private: bool,
    /// Parent channel of the thread.
    ///
    /// This field can be [`None`] if the parent channel has been
    /// deleted.
    pub parent_id: Option<Id<ChannelMarker>>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u64>,
}

impl From<CachedThread> for CachedChannel {
    fn from(thread: CachedThread) -> Self {
        CachedChannel::Thread(thread)
    }
}
