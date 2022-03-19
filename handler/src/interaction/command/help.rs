//! Help command.

use raidprotect_model::ClusterState;
use thiserror::Error;
use tracing::{error, instrument};
use twilight_embed_builder::{EmbedBuilder, EmbedError};
use twilight_interactions::{
    command::{CommandModel, CreateCommand, CreateOption},
    error::ParseError,
};

use crate::interaction::{
    context::CommandContext,
    response::{CommandResponse, InteractionError, InteractionErrorKind},
};

/// Help command model.
#[derive(Debug, CommandModel, CreateCommand)]
#[command(name = "help", desc = "Show the list of available commands")]
pub struct HelpCommand {
    /// Displays the help for a specific command.
    pub command: Option<String>,
}

/// Command list model.
#[derive(CreateOption)]
pub enum Command {
    #[option(name = "test", value = "test")]
    Test,
}

impl HelpCommand {
    /// Handle interaction for this command.
    #[instrument]
    pub async fn handle(
        context: CommandContext,
        _state: &ClusterState,
    ) -> Result<CommandResponse, HelpCommandError> {
        let _parsed = HelpCommand::from_interaction(context.data.into())?;

        let embed = EmbedBuilder::new().description("Hello world!").build()?;

        Ok(CommandResponse::Embed(embed))
    }
}

/// Error when executing [`HelpCommand`]
#[derive(Debug, Error)]
pub enum HelpCommandError {
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("failed to build embed: {0}")]
    Embed(#[from] EmbedError),
}

impl InteractionError for HelpCommandError {
    const INTERACTION_NAME: &'static str = "help";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            HelpCommandError::Parse(error) => InteractionErrorKind::internal(error),
            HelpCommandError::Embed(error) => InteractionErrorKind::internal(error),
        }
    }
}
