//! Help command.

use raidprotect_gateway::event::context::GuildContext;
use thiserror::Error;
use tracing::{error, instrument};
use twilight_embed_builder::{EmbedBuilder, EmbedError};
use twilight_interactions::command::{CommandModel, CreateCommand, CreateOption};
use twilight_model::{
    application::{callback::CallbackData, interaction::Interaction},
    channel::embed::Embed,
};

use crate::embed;

use super::IntoCallback;

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
        _interaction: Interaction,
        _ctx: GuildContext,
    ) -> Result<Embed, HelpCommandError> {
        let embed = EmbedBuilder::new().description("Hello world!").build()?;

        Ok(embed)
    }
}

/// Error when executing [`HelpCommand`]
#[derive(Debug, Error)]
pub enum HelpCommandError {
    #[error("failed to build embed: {0}")]
    Embed(#[from] EmbedError),
}

impl IntoCallback for HelpCommandError {
    fn into_callback(self) -> CallbackData {
        match self {
            error => {
                error!(error = %error, "error occured when handling `help` command");
                embed::error::internal_error().into_callback()
            }
        }
    }
}
