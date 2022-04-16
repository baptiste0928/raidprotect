//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use thiserror::Error;
use tracing::instrument;
use twilight_interactions::{
    command::{CommandModel, CreateCommand, ResolvedUser},
    error::ParseError,
};

use crate::{
    embed,
    interaction::{
        context::CommandContext,
        response::{CommandResponse, InteractionError, InteractionErrorKind},
    },
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
    pub async fn handle(context: CommandContext) -> Result<CommandResponse, KickCommandError> {
        let parsed = KickCommand::from_interaction(context.data.into())?;

        let _member = parsed.user.member.ok_or(KickCommandError::MissingMember {
            user: parsed.user.resolved.name,
        })?;

        todo!()
    }
}

/// Error when executing [`KickCommand`]
#[derive(Debug, Error)]
pub enum KickCommandError {
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("user is not a guild member")]
    MissingMember { user: String },
}

impl InteractionError for KickCommandError {
    const INTERACTION_NAME: &'static str = "kick";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            KickCommandError::Parse(error) => InteractionErrorKind::internal(error),
            KickCommandError::MissingMember { user } => embed::kick::not_member(user).into(),
        }
    }
}
