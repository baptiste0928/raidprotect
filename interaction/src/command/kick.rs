//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use raidprotect_cache::{permission::PermissionError, redis::RedisClientError};
use raidprotect_state::ClusterState;
use raidprotect_translations::Lang;
use raidprotect_util::text::TextProcessExt;
use thiserror::Error;
use tracing::instrument;
use twilight_interactions::{
    command::{CommandModel, CreateCommand, ResolvedUser},
    error::ParseError,
};
use twilight_model::{
    application::{
        component::{text_input::TextInputStyle, ActionRow, Component, TextInput},
        interaction::application_command::CommandData,
    },
    guild::Permissions,
};

use crate::{
    context::InteractionContext,
    embed,
    response::{InteractionError, InteractionErrorKind, InteractionResponse},
};

/// Kick command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "kick",
    desc = "Kicks a user from the server",
    default_permissions = "KickCommand::default_permissions",
    dm_permission = false
)]
pub struct KickCommand {
    /// Member to kick.
    #[command(rename = "member")]
    pub user: ResolvedUser,
    /// Reason for kick.
    pub reason: Option<String>,
}

impl KickCommand {
    fn default_permissions() -> Permissions {
        Permissions::KICK_MEMBERS
    }

    #[instrument]
    pub async fn handle(
        context: InteractionContext<CommandData>,
        state: &ClusterState,
    ) -> Result<InteractionResponse, KickCommandError> {
        let parsed = KickCommand::from_interaction(context.data.into())?;
        let guild = &context.guild.ok_or(KickCommandError::GuildOnly)?;

        let user = parsed.user.resolved;
        let member = parsed.user.member.ok_or(KickCommandError::MissingMember {
            user: user.name.clone(),
        })?;

        // Fetch the author and the bot permissions.
        let permissions = state.redis().permissions(guild.guild.id).await?;
        let author_permissions = permissions
            .member(context.user.id, &guild.member.roles)
            .await?;
        let member_permissions = permissions.member(user.id, &member.roles).await?;
        let bot_permissions = permissions.current_member().await?;

        // Check if the author and the bot have required permissions.
        if member_permissions.is_owner() {
            return Err(KickCommandError::MemberOwner);
        }

        if !bot_permissions.guild().contains(Permissions::KICK_MEMBERS) {
            return Err(KickCommandError::BotMissingKickPermission);
        }

        // Check if the role hierarchy allow the author and the bot to perform
        // the kick.
        let member_highest_role = member_permissions.highest_role();

        if member_highest_role >= author_permissions.highest_role() {
            return Err(KickCommandError::UserHierarchy);
        }

        if member_highest_role >= bot_permissions.highest_role() {
            return Err(KickCommandError::BotHierarchy);
        }

        // Send reason modal.
        match parsed.reason {
            Some(_reason) => Ok(InteractionResponse::EphemeralDeferredMessage),
            None => KickCommand::reason_modal(user.name, guild.config().enforce_reason),
        }
    }

    /// Modal that asks the user to enter a reason for the kick.
    ///
    /// This modal is only shown if the user has not specified a reason in the
    /// initial command.
    fn reason_modal(
        username: String,
        enforce_reason: bool,
    ) -> Result<InteractionResponse, KickCommandError> {
        let components = vec![
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "reason".to_string(),
                    label: Lang::Fr.modal_kick_reason_label().to_string(),
                    max_length: Some(100),
                    min_length: None,
                    placeholder: Some(Lang::Fr.modal_reason_placeholder().to_string()),
                    required: Some(enforce_reason),
                    style: TextInputStyle::Short,
                    value: None,
                })],
            }),
            Component::ActionRow(ActionRow {
                components: vec![Component::TextInput(TextInput {
                    custom_id: "notes".to_string(),
                    label: Lang::Fr.modal_notes_label().to_string(),
                    max_length: Some(1000),
                    min_length: None,
                    placeholder: Some(Lang::Fr.modal_notes_placeholder().to_string()),
                    required: Some(false),
                    style: TextInputStyle::Paragraph,
                    value: None,
                })],
            }),
        ];

        Ok(InteractionResponse::Modal {
            custom_id: "kick_reason_modal".to_string(),
            title: Lang::Fr.modal_kick_title(username.truncate(15)),
            components,
        })
    }
}

/// Error when executing [`KickCommand`]
#[derive(Debug, Error)]
pub enum KickCommandError {
    #[error("command must be run in a guild")]
    GuildOnly,
    #[error("user is not a guild member")]
    MissingMember { user: String },
    #[error("bot has not the kick permission")]
    BotMissingKickPermission,
    #[error("member is the owner of the guild")]
    MemberOwner,
    #[error("member has a role above the author")]
    UserHierarchy,
    #[error("member has a role above the bot")]
    BotHierarchy,
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("unable to get permissions from cache")]
    Permission(#[from] PermissionError),
    #[error(transparent)]
    Redis(#[from] RedisClientError),
}

impl InteractionError for KickCommandError {
    const INTERACTION_NAME: &'static str = "kick";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            KickCommandError::GuildOnly => InteractionErrorKind::internal(self),
            KickCommandError::MissingMember { user } => embed::kick::not_member(user).into(),
            KickCommandError::BotMissingKickPermission => {
                embed::kick::bot_missing_permission().into()
            }
            KickCommandError::MemberOwner => embed::kick::member_owner().into(),
            KickCommandError::UserHierarchy => embed::kick::user_hierarchy().into(),
            KickCommandError::BotHierarchy => embed::kick::bot_hierarchy().into(),
            KickCommandError::Parse(error) => InteractionErrorKind::internal(error),
            KickCommandError::Permission(error) => InteractionErrorKind::internal(error),
            KickCommandError::Redis(error) => InteractionErrorKind::internal(error),
        }
    }
}
