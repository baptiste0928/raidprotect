//! State for interactions (buttons, select menus, modals).

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use time::OffsetDateTime;
use twilight_model::{
    http::interaction::InteractionResponseData,
    id::{
        marker::{GuildMarker, InteractionMarker, UserMarker},
        Id,
    },
    user::User,
};

use crate::{
    cache::RedisModel,
    database::model::ModlogType,
    serde::{DateTimeAsI64, IdAsU64},
};

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

/// State for a pending captcha.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingCaptcha {
    /// Id of the guild.
    #[serde_as(as = "IdAsU64")]
    pub guild_id: Id<GuildMarker>,
    /// Id of the member that needs to solve the captcha.
    #[serde_as(as = "IdAsU64")]
    pub member_id: Id<UserMarker>,
    /// Code of the captcha.
    pub code: String,
    /// Number of time the captcha has been regenerated.
    ///
    /// This number is incremented each time the user clicks on the "start
    /// verification" button (new image with the same code) or the "regenerate"
    /// button (new image with a new code).
    pub regenerate_count: u8,
    /// Time at which the captcha expires.
    #[serde_as(as = "DateTimeAsI64")]
    pub expires_at: OffsetDateTime,
}

impl RedisModel for PendingCaptcha {
    type Id = (Id<GuildMarker>, Id<UserMarker>);

    // Captcha expires after 5 minutes, but we add some margin since the presence
    // of this entry is used to check if the captcha has been solved.
    const EXPIRES_AFTER: Option<usize> = Some(10 * 60);

    fn key(&self) -> String {
        Self::key_from(&(self.guild_id, self.member_id))
    }

    fn key_from(id: &Self::Id) -> String {
        format!(
            "pending:captcha:{guild}:{member}",
            guild = id.0.get(),
            member = id.1.get()
        )
    }
}

/// State for a pending sanction modal.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSanction {
    /// Initial interaction ID.
    #[serde_as(as = "IdAsU64")]
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
