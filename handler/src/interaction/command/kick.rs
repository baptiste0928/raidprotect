//! Kick command.
//!
//! The command allows to kick a member from the server. User can specify a
//! reason directly in the command (as an optional parameter), or in the modal
//! that is shown if it hasn't been set in the command.
//!
//! When a user is kicked, the action is logged in the database and a message is
//! sent in the guild's logs channel. The kicked user receives a pm with the
//! reason of the kick.

use tracing::instrument;
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};

use crate::interaction::{context::CommandContext, response::CommandResponse};

/// Kick command model.
///
/// See the [`module`][self] documentation for more information.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "kick", desc = "Kicks a user from the server")]
pub struct KickCommand {
    /// Member to kick.
    pub member: ResolvedUser,
    /// Reason for kick.
    pub reason: Option<String>,
}

impl KickCommand {
    #[instrument]
    pub async fn handle(context: CommandContext) -> CommandResponse {
        todo!()
    }
}
