//! Help command.

use raidprotect_model::ClusterState;
use thiserror::Error;
use tracing::{error, instrument};
use twilight_embed_builder::{EmbedBuilder, EmbedError};
use twilight_interactions::{
    command::{CommandModel, CreateCommand, CreateOption},
    error::ParseError,
};
use twilight_model::{application::interaction::ApplicationCommand, channel::embed::Embed};

use crate::interaction::response::{InteractionError, InteractionErrorData};

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
        command: ApplicationCommand,
        _state: &ClusterState,
    ) -> Result<Embed, HelpCommandError> {
        let _parsed = HelpCommand::from_interaction(command.data.into())?;

        let embed = EmbedBuilder::new().description("Hello world!").build()?;

        Ok(embed)
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
    fn into_error(self) -> InteractionErrorData {
        match self {
            HelpCommandError::Parse(error) => InteractionErrorData::internal(Some("help"), error),
            HelpCommandError::Embed(error) => InteractionErrorData::internal(Some("help"), error),
        }
    }
}
