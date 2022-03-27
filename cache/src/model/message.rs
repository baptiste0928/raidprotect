use twilight_model::{
    channel::Attachment,
    datetime::Timestamp,
    id::{
        marker::{ChannelMarker, MessageMarker, RoleMarker, UserMarker},
        Id,
    },
};
use url::Url;

/// Cached model of a [`Message`].
///
/// This model is used with [`MessageCache`].
///
/// [`Message`]: twilight_model::channel::message::Message
/// [`MessageCache`]: crate::MessageCache
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedMessage {
    /// ID of the message.
    pub id: Id<MessageMarker>,
    /// Message author id.
    pub author_id: Id<UserMarker>,
    /// Message channel id.
    pub channel_id: Id<ChannelMarker>,
    /// Message content.
    pub content: String,
    /// Timestamp of when the message was created.
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
    pub mention_users: Vec<Id<UserMarker>>,
    /// List of roles mentioned in the message.
    pub mention_roles: Vec<Id<RoleMarker>>,
}

/// Kind of message link.
///
/// This type is used in [`CachedMessage`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLink {
    /// Invite URL
    Invite(Url),
    /// Media URL (image or video)
    Media(Url),
    /// URL that does not belong to one of the previous categories
    Other(Url),
}
