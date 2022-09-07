use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};
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
#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedChannel {
    /// Id of the channel
    #[serde_as(as = "IdAsU64")]
    pub id: Id<ChannelMarker>,
    /// Id of the guild to which the channel belongs.
    #[serde_as(as = "IdAsU64")]
    pub guild_id: Id<GuildMarker>,
    /// Type of the channel.
    pub kind: ChannelType,
    /// Name of the channel.
    pub name: String,
    /// Id of the parent channel.
    ///
    /// For guild channels this is the ID of the parent category channel.
    ///
    /// For threads this is the ID of the channel the thread was created in.
    #[serde_as(as = "Option<IdAsU64>")]
    pub parent_id: Option<Id<ChannelMarker>>,
    /// Permission overwrites of the channel.
    ///
    /// This field is not present on thread channels.
    pub permission_overwrites: Option<Vec<PermissionOverwrite>>,
    /// Sorting position of the category.
    ///
    /// This field is not present on thread channels.
    pub position: Option<i32>,
    /// Amount of seconds a user has to wait between two message.
    pub rate_limit_per_user: Option<u16>,
}

impl CachedChannel {
    /// Whether the channel is a thread channel.
    pub fn is_thread(&self) -> bool {
        [
            ChannelType::GuildPublicThread,
            ChannelType::GuildPrivateThread,
            ChannelType::GuildNewsThread,
        ]
        .contains(&self.kind)
    }

    /// Whether a [`ChannelType`] can be cached with this model.
    pub(crate) fn is_cached(kind: ChannelType) -> bool {
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
        Self::key_from(&self.id)
    }

    fn key_from(id: &Self::Id) -> String {
        format!("c:channel:{id}")
    }
}
