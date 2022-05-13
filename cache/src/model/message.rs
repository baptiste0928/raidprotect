//! Messages cache models.

use raidprotect_model::serde::{IdAsU64, TimestampAsI64};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    channel::Attachment,
    datetime::Timestamp,
    id::{
        marker::{ChannelMarker, MessageMarker, RoleMarker, UserMarker},
        Id,
    },
};
use url::Url;

use crate::redis::RedisModel;

/// Cached model of a [`Message`].
///
/// [`Message`]: twilight_model::channel::message::Message
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedMessage {
    /// ID of the message.
    #[serde_as(as = "IdAsU64")]
    pub id: Id<MessageMarker>,
    /// Message author id.
    #[serde_as(as = "IdAsU64")]
    pub author_id: Id<UserMarker>,
    /// Message channel id.
    #[serde_as(as = "IdAsU64")]
    pub channel_id: Id<ChannelMarker>,
    /// Message content.
    pub content: String,
    /// Timestamp of when the message was created.
    #[serde_as(as = "TimestampAsI64")]
    pub timestamp: Timestamp,
    /// List of message words.
    ///
    /// The words are split according to the Unicode specification and each
    /// character is converted into ASCII.
    pub words: Vec<String>,
    /// List of message attachments.
    pub attachments: Vec<Attachment>,
    /// List of links included in the message.
    pub links: Vec<MessageLink>,
    /// Whether the message mentions everyone (@everyone or @here mentions)
    pub mention_everyone: bool,
    /// List of users mentioned in the message.
    #[serde_as(as = "Vec<IdAsU64>")]
    pub mention_users: Vec<Id<UserMarker>>,
    /// List of roles mentioned in the message.
    #[serde_as(as = "Vec<IdAsU64>")]
    pub mention_roles: Vec<Id<RoleMarker>>,
}

impl RedisModel for CachedMessage {
    type Id = Id<MessageMarker>;

    // Message expiration duration (2 minutes)
    const EXPIRES_AFTER: Option<usize> = Some(2 * 60);

    fn key(&self) -> String {
        Self::key_from(&self.id)
    }

    fn key_from(id: &Self::Id) -> String {
        format!("c:msg:{id}")
    }
}

/// Kind of message link.
///
/// This type is used in [`CachedMessage`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum MessageLink {
    /// Invite URL
    Invite(Url),
    /// Media URL (image or video)
    Media(Url),
    /// URL that does not belong to one of the previous categories
    Other(Url),
}
