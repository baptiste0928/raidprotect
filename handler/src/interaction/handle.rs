use std::sync::Arc;

use raidprotect_model::ClusterState;
use tracing::warn;
use twilight_model::application::interaction::ApplicationCommand;

use crate::embed;

use super::{
    command,
    response::{CommandResponder, IntoResponse},
};

/// Handle incoming [`ApplicationCommand`]
///
/// This method will handle incoming commands depending on whereas they can run
/// on both dms and guilds, or only on guild.
pub async fn handle_command(command: ApplicationCommand, state: Arc<ClusterState>) {
    let responder = CommandResponder::from_command(&command);

    let response = match &*command.data.name {
        "help" => command::help::HelpCommand::handle(command, &state)
            .await
            .into_response(),
        name => {
            warn!(name = name, "unknown command received");

            embed::error::unknown_command().into_response()
        }
    };

    responder.respond(&state, response).await;
}
