//! State for interactions (buttons, select menus, modals).

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    http::interaction::InteractionResponseData,
    id::{marker::UserMarker, Id},
    user::User,
};

use crate::{cache::RedisModel, mongodb::modlog::ModlogType, serde::IdAsU64};

/// State of a component (button, select modal) waiting for user interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingComponent {
    PostInChat(PostInChatButton),
}

impl PendingComponent {
    /// Get the component unique identifier.
    pub fn id(&self) -> &str {
        match self {
            Self::PostInChat(component) => &component.id,
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

// State of a modal waiting for user interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PendingModal {
    Sanction(PendingSanction),
}

impl PendingModal {
    /// Get the component unique identifier.
    pub fn id(&self) -> &str {
        match self {
            Self::Sanction(component) => &component.id,
        }
    }
}

impl RedisModel for PendingModal {
    type Id = str;

    // Pending modals expires after 5 minutes
    const EXPIRES_AFTER: Option<usize> = Some(5 * 60);

    fn key(&self) -> String {
        Self::key_from(self.id())
    }

    fn key_from(id: &Self::Id) -> String {
        format!("pending:modal:{id}")
    }
}

/// State for a pending sanction modal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSanction {
    /// Component unique identifier.
    pub id: String,
    /// Type of the pending modlog.
    pub kind: ModlogType,
    /// User targeted by the sanction.
    pub user: User,
}
