use std::sync::Arc;

use anyhow::Context;
use raidprotect_model::cache::model::component::PendingComponent;
use tracing::{error, warn};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::{
        command::Command,
        interaction::{ApplicationCommand, MessageComponentInteraction},
    },
    id::{marker::ApplicationMarker, Id},
};

use super::{
    command::{help::HelpCommand, profile::ProfileCommand},
    component::PostInChat,
    context::InteractionContext,
    response::InteractionResponder,
};
use crate::{
    cluster::ClusterState,
    interaction::{command::kick::KickCommand, embed},
};

/// Handle incoming [`ApplicationCommand`]
///
/// This method will handle incoming commands depending on whereas they can run
/// on both dms and guilds, or only on guild.
pub async fn handle_command(command: ApplicationCommand, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_command(&command);
    let context = InteractionContext::from_command(command, &state)
        .await
        .context("failed to create command context");

    let response = match context {
        Ok(context) => match &*context.data.name {
            "help" => HelpCommand::handle(context).await,
            "profile" => ProfileCommand::handle(context, &state).await,
            "kick" => KickCommand::handle(context, &state).await,
            name => {
                warn!(name = name, "unknown command received");
                Ok(embed::error::unknown_command())
            }
        },
        Err(e) => Err(e),
    };

    match response {
        Ok(response) => responder.respond(&state, response).await,
        Err(error) => {
            error!(error = ?error, "error while processing command");

            responder
                .respond(&state, embed::error::internal_error())
                .await;
        }
    };
}

/// Register commands to the Discord API.
pub async fn register_commands(state: &ClusterState, application_id: Id<ApplicationMarker>) {
    let commands: Vec<Command> = vec![
        HelpCommand::create_command().into(),
        ProfileCommand::create_command().into(),
        KickCommand::create_command().into(),
    ];

    let client = state.http().interaction(application_id);

    if let Err(error) = client.set_global_commands(&commands).exec().await {
        error!(error = ?error, "failed to register commands");
    }
}

/// Handle incoming [`MessageComponentInteraction`].
pub async fn handle_component(component: MessageComponentInteraction, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_component(&component);
    let context = InteractionContext::from_component(component, &state)
        .await
        .context("failed to create component context");

    let component = match context {
        Ok(context) => state
            .redis()
            .get::<PendingComponent>(&context.data.custom_id)
            .await
            .context("failed to fetch component state"),
        Err(e) => Err(e),
    };

    let response = match component {
        Ok(Some(component)) => match component {
            PendingComponent::PostInChatButton(c) => PostInChat::handle(c),
            PendingComponent::Sanction(_) => unimplemented!(),
        },
        Ok(None) => embed::error::expired_component(),
        Err(error) => {
            error!(error = ?error, "error while processing component");
            embed::error::internal_error()
        }
    };

    responder.respond(&state, response).await;
}
