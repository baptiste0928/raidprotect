use std::sync::Arc;

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
    component::post_in_chat::PostInChat,
    context::InteractionContext,
    response::{InteractionResponder, IntoResponse},
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
    let context = match InteractionContext::from_command(command, &state).await {
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
        "help" => HelpCommand::handle(context).await.into_response(),
        "profile" => ProfileCommand::handle(context, &state)
            .await
            .into_response(),
        "kick" => KickCommand::handle(context, &state).await.into_response(),
        name => {
            warn!(name = name, "unknown command received");

            embed::error::unknown_command().into_response()
        }
    };

    responder.respond(&state, response).await;
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
        error!(error = %error, "failed to register commands");
    }
}

/// Handle incoming [`MessageComponentInteraction`].
pub async fn handle_component(component: MessageComponentInteraction, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_component(&component);
    let context = match InteractionContext::from_component(component, &state).await {
        Ok(context) => context,
        Err(error) => {
            warn!(error = %error, "failed to create component context");
            responder
                .respond(&state, embed::error::internal_error().into_response())
                .await;

            return;
        }
    };

    let pending_component = match state
        .redis()
        .get::<PendingComponent>(&context.data.custom_id)
        .await
    {
        Ok(component) => component,
        Err(error) => {
            error!(error = %error, "failed to fetch component state");
            responder
                .respond(&state, embed::error::internal_error().into_response())
                .await;

            return;
        }
    };

    let response = if let Some(component) = pending_component {
        match component {
            PendingComponent::PostInChatButton(component) => PostInChat::handle(component),
        }
    } else {
        embed::error::expired_component().into_response()
    };

    responder.respond(&state, response).await;
}
