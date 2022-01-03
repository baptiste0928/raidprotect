use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite,
    id::{ChannelId, GuildId},
};

use super::partial::{
    IntoPartial, PartialCategoryChannel, PartialChannel, PartialTextChannel, PartialThread,
};

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

impl IntoPartial for CachedChannel {
    type Partial = PartialChannel;

    fn into_partial(&self) -> Self::Partial {
        match self {
            CachedChannel::Text(channel) => PartialChannel::Text(channel.into_partial()),
            CachedChannel::Category(channel) => PartialChannel::Category(channel.into_partial()),
            CachedChannel::Thread(channel) => PartialChannel::Thread(channel.into_partial()),
        }
    }
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

impl IntoPartial for CachedTextChannel {
    type Partial = PartialTextChannel;

    fn into_partial(&self) -> Self::Partial {
        PartialTextChannel {
            parent_id: self.parent_id,
            permission_overwrites: self.permission_overwrites.clone(),
        }
    }
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

impl IntoPartial for CachedCategoryChannel {
    type Partial = PartialCategoryChannel;

    fn into_partial(&self) -> Self::Partial {
        PartialCategoryChannel {
            permission_overwrites: self.permission_overwrites.clone(),
        }
    }
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

impl IntoPartial for CachedThread {
    type Partial = PartialThread;

    fn into_partial(&self) -> Self::Partial {
        PartialThread {
            parent_id: self.parent_id,
        }
    }
}
