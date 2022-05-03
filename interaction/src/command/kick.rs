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
use twilight_model::{
    application::interaction::application_command::CommandData, guild::Permissions,
};

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
        let guild = &context.guild.ok_or(KickCommandError::GuildOnly)?;

        let user = parsed.user.resolved;
        let member = parsed
            .user
            .member
            .ok_or(KickCommandError::MissingMember { user: user.name })?;

        // Fetch the author and the bot permissions.
        let author_permissions = state
            .redis()
            .permissions(guild.guild.id, context.user.id, &guild.member.roles)
            .await?
            .ok_or(KickCommandError::PermissionNotFound)?;

        let member_permissions = state
            .redis()
            .permissions(guild.guild.id, user.id, &member.roles)
            .await?
            .ok_or(KickCommandError::PermissionNotFound)?;

        let bot_permissions = state
            .redis()
            .current_member_permissions(guild.id)
            .await?
            .ok_or(KickCommandError::PermissionNotFound)?;

        // Check if the author and the bot have required permissions.
        if member_permissions.is_owner() {
            return Err(KickCommandError::MemberOwner);
        }

        if !author_permissions
            .guild()
            .contains(Permissions::KICK_MEMBERS)
        {
            return Err(KickCommandError::MissingKickPermission);
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
            KickCommandError::BotMissingKickPermission => {
                embed::kick::bot_missing_permission().into()
            }
            KickCommandError::MemberOwner => embed::kick::member_owner().into(),
            KickCommandError::UserHierarchy => embed::kick::user_hierarchy().into(),
            KickCommandError::BotHierarchy => embed::kick::bot_hierarchy().into(),
            KickCommandError::Parse(error) => InteractionErrorKind::internal(error),
            KickCommandError::PermissionNotFound => InteractionErrorKind::internal(self),
            KickCommandError::Redis(error) => InteractionErrorKind::internal(error),
        }
    }
}
