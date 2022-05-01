//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use raidprotect_cache::redis::RedisClientError;
use raidprotect_model::interaction::InteractionResponse;
use raidprotect_state::ClusterState;
use thiserror::Error;
use tracing::instrument;
use twilight_interactions::{
    command::{CommandModel, CreateCommand, ResolvedUser},
    error::ParseError,
};
use twilight_model::{application::interaction::application_command::CommandData, guild::Permissions};

use crate::{
    context::InteractionContext,
    embed,
    response::{InteractionError, InteractionErrorKind},
};

/// Kick command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "kick", desc = "Kicks a user from the server")]
pub struct KickCommand {
    /// Member to kick.
    #[command(rename = "member")]
    pub user: ResolvedUser,
    /// Reason for kick.
    pub reason: Option<String>,
}

impl KickCommand {
    #[instrument]
    pub async fn handle(
        context: InteractionContext<CommandData>,
        state: &ClusterState,
    ) -> Result<InteractionResponse, KickCommandError> {
        let parsed = KickCommand::from_interaction(context.data.into())?;
        let guild_context = &context.guild.ok_or(KickCommandError::GuildOnly)?;

        let user = parsed.user.resolved;
        let _member = parsed
            .user
            .member
            .ok_or(KickCommandError::MissingMember { user: user.name })?;

        // Check member and bot permissions
        let author_permissions = state
            .redis()
            .permissions(
                guild_context.guild.id,
                context.user.id,
                &guild_context.member.roles,
            )
            .await?
            .ok_or(KickCommandError::PermissionNotFound)?;

        if !author_permissions
            .guild()
            .contains(Permissions::KICK_MEMBERS)
        {
            return Err(KickCommandError::MissingKickPermission);
        }

        todo!()
    }
}

/// Error when executing [`KickCommand`]
#[derive(Debug, Error)]
pub enum KickCommandError {
    #[error("command must be run in a guild")]
    GuildOnly,
    #[error("user is not a guild member")]
    MissingMember { user: String },
    #[error("user has not the kick permission")]
    MissingKickPermission,
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("unable to get permissions from cache")]
    PermissionNotFound,
    #[error(transparent)]
    Redis(#[from] RedisClientError),
}

impl InteractionError for KickCommandError {
    const INTERACTION_NAME: &'static str = "kick";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            KickCommandError::GuildOnly => embed::error::guild_only().into(),
            KickCommandError::MissingMember { user } => embed::kick::not_member(user).into(),
            KickCommandError::MissingKickPermission => embed::kick::missing_permission().into(),
            KickCommandError::Parse(error) => InteractionErrorKind::internal(error),
            KickCommandError::PermissionNotFound => InteractionErrorKind::internal(self),
            KickCommandError::Redis(error) => InteractionErrorKind::internal(error),
        }
    }
}
