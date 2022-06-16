//! Help command.

use tracing::instrument;
use twilight_interactions::command::{CommandModel, CommandOption, CreateCommand, CreateOption};
use twilight_model::application::interaction::application_command::CommandData;
use twilight_util::builder::embed::EmbedBuilder;

use crate::interaction::{context::InteractionContext, response::InteractionResponse};

/// Help command model.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Show the list of available commands",
    dm_permission = true
)]
pub struct HelpCommand {
    /// Displays the help for a specific command.
    pub command: Option<Command>,
}

/// Command list model.
#[derive(Debug, Clone, CommandOption, CreateOption)]
pub enum Command {
    #[option(name = "test", value = "test")]
    Test,
}

impl HelpCommand {
    /// Handle interaction for this command.
    #[instrument]
    pub async fn handle(
        context: InteractionContext<CommandData>,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let _parsed = HelpCommand::from_interaction(context.data.into())?;

        let embed = EmbedBuilder::new().description("Hello world!").build();

        Ok(InteractionResponse::Embed(embed))
    }
}
