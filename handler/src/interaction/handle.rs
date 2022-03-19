use std::sync::Arc;

use raidprotect_model::ClusterState;
use tracing::{error, warn};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::{command::Command, interaction::ApplicationCommand},
    id::{marker::ApplicationMarker, Id},
};

use crate::embed;

use super::{
    command::{self, help::HelpCommand},
    context::CommandContext,
    response::{CommandResponder, IntoResponse},
};

/// Handle incoming [`ApplicationCommand`]
///
/// This method will handle incoming commands depending on whereas they can run
/// on both dms and guilds, or only on guild.
pub async fn handle_command(command: ApplicationCommand, state: Arc<ClusterState>) {
    let responder = CommandResponder::from_command(&command);
    let context = match CommandContext::from_command(command) {
        Ok(context) => context,
        Err(error) => {
            warn!(error = %error, "failed to create command context");
            responder
                .respond(&state, embed::error::internal_error().into_response())
                .await;

            return;
        }
    };

    let response = match &*context.data.name {
        "help" => HelpCommand::handle(context, &state).await.into_response(),
        name => {
            warn!(name = name, "unknown command received");

            embed::error::unknown_command().into_response()
        }
    };

    responder.respond(&state, response).await;
}

/// Register commands to the Discord API.
///
/// The commands will be registered globally unless a `command_guild` is set.
pub async fn register_commands(
    state: &ClusterState,
    application_id: Id<ApplicationMarker>,
    command_guild: Option<u64>,
) {
    let commands: Vec<Command> = vec![command::help::HelpCommand::create_command().into()];

    let client = state.http().interaction(application_id);

    let result = match command_guild {
        Some(id) => {
            client
                .set_guild_commands(Id::new(id), &commands)
                .exec()
                .await
        }
        None => client.set_global_commands(&commands).exec().await,
    };

    if let Err(error) = result {
        error!(error = %error, "failed to register commands");
    }
}