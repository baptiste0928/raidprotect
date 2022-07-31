//! State for interactions (buttons, select menus, modals).

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::{
    http::interaction::InteractionResponseData,
    id::{
        marker::{InteractionMarker, UserMarker},
        Id,
    },
    user::User,
};

use crate::{cache::RedisModel, mongodb::modlog::ModlogType, serde::IdAsU64};

/// State for the "post in chat" button.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostInChatButton {
    /// Response to send to the channel.
    pub response: InteractionResponseData,
    /// Initial interaction ID.
    pub interaction_id: Id<InteractionMarker>,
    /// Id of the initial interaction author.
    #[serde_as(as = "IdAsU64")]
    pub author_id: Id<UserMarker>,
}

impl RedisModel for PostInChatButton {
    type Id = str;

    // Post in chat buttons expires after 5 minutes
    const EXPIRES_AFTER: Option<usize> = Some(5 * 60);

    fn key(&self) -> String {
        Self::key_from(&self.interaction_id.to_string())
    }

    fn key_from(id: &Self::Id) -> String {
        format!("pending:post-in-chat:{id}")
    }
}

/// State for a pending sanction modal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSanction {
    /// Initial interaction ID.
    pub interaction_id: Id<InteractionMarker>,
    /// Type of the pending modlog.
    pub kind: ModlogType,
    /// User targeted by the sanction.
    pub user: User,
}

impl RedisModel for PendingSanction {
    type Id = str;

    // Pending modals expires after 5 minutes
    const EXPIRES_AFTER: Option<usize> = Some(5 * 60);

    fn key(&self) -> String {
        Self::key_from(&self.interaction_id.to_string())
    }

    fn key_from(id: &Self::Id) -> String {
        format!("pending:sanction:{id}")
    }
}
