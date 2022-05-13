//! State for message component interactions (buttons, select menus).

use raidprotect_model::serde::IdAsU64;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    http::interaction::InteractionResponseData,
    id::{marker::UserMarker, Id},
};

use crate::redis::RedisModel;

/// State of a component waiting for user interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingComponent {
    PostInChatButton(PostInChatButton),
}

impl PendingComponent {
    /// Get the component unique identifier.
    pub fn id(&self) -> &str {
        match self {
            PendingComponent::PostInChatButton(component) => &component.id,
        }
    }
}

impl RedisModel for PendingComponent {
    type Id = str;

    // Pending components expires after 5 minutes
    const EXPIRES_AFTER: Option<usize> = Some(5 * 60);

    fn key(&self) -> String {
        Self::key_from(self.id())
    }

    fn key_from(id: &Self::Id) -> String {
        format!("pending:component:{id}")
    }
}

/// State for the "post in chat" button.
///
/// See the `raidprotect-handler` for more information.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInChatButton {
    /// Component unique identifier.
    pub id: String,
    /// Response to send to the channel.
    pub response: InteractionResponseData,
    /// Id of the initial interaction author.
    #[serde_as(as = "IdAsU64")]
    pub author_id: Id<UserMarker>,
}
