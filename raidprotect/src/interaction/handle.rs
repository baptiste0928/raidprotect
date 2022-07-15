use std::sync::Arc;

use anyhow::{bail, Context};
use raidprotect_model::cache::model::interaction::{PendingComponent, PendingModal};
use tracing::{debug, error, warn};
use twilight_interactions::command::CreateCommand;
use twilight_model::{
    application::{
        command::Command,
        interaction::{Interaction, InteractionData, InteractionType},
    },
    id::{marker::ApplicationMarker, Id},
};

use super::{
    command::{moderation::KickCommand, profile::ProfileCommand},
    component::PostInChat,
    embed,
    response::{InteractionResponder, InteractionResponse},
};
use crate::cluster::ClusterState;

/// Handle incoming [`Interaction`].
pub async fn handle_interaction(interaction: Interaction, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_interaction(&interaction);
    debug!("received {} interaction", interaction.kind.kind());

    let response = match interaction.kind {
        InteractionType::ApplicationCommand => handle_command(interaction, &state).await,
        InteractionType::MessageComponent => handle_component(interaction, &state).await,
        InteractionType::ModalSubmit => handle_modal(interaction, &state).await,
        other => {
            warn!("received unexpected {} interaction", other.kind());

            return;
        }
    };

    match response {
        Ok(response) => responder.respond(&state, response).await,
        Err(error) => {
            error!(error = ?error, "error while processing interaction");

            responder
                .respond(&state, embed::error::internal_error())
                .await;
        }
    }
}

/// Handle incoming command interaction.
async fn handle_command(
    interaction: Interaction,
    state: &ClusterState,
) -> Result<InteractionResponse, anyhow::Error> {
    let name = match &interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => &*data.name,
        _ => bail!("expected application command data"),
    };

    match name {
        "profile" => ProfileCommand::handle(interaction, state).await,
        "kick" => KickCommand::handle(interaction, state).await,
        name => {
            warn!(name = name, "received unknown command");

            Ok(embed::error::unknown_command())
        }
    }
}

/// Handle incoming component interaction
async fn handle_component(
    interaction: Interaction,
    state: &ClusterState,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::MessageComponent(data)) => &*data.custom_id,
        _ => bail!("expected message component data"),
    };

    let component = match state
        .redis()
        .get::<PendingComponent>(custom_id)
        .await
        .context("failed to get component state")?
    {
        Some(component) => component,
        None => return Ok(embed::error::expired_interaction()),
    };

    match component {
        PendingComponent::PostInChat(component) => Ok(PostInChat::handle(component)),
    }
}

/// Handle incoming modal interaction
async fn handle_modal(
    interaction: Interaction,
    state: &ClusterState,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::ModalSubmit(data)) => &*data.custom_id,
        _ => bail!("expected modal submit data"),
    };

    let modal = match state
        .redis()
        .get::<PendingModal>(custom_id)
        .await
        .context("failed to get modal state")?
    {
        Some(modal) => modal,
        None => return Ok(embed::error::expired_interaction()),
    };

    match modal {
        PendingModal::Sanction(_) => bail!("not implemented"),
    }
}

/// Register commands to the Discord API.
pub async fn register_commands(state: &ClusterState, application_id: Id<ApplicationMarker>) {
    let commands: Vec<Command> = vec![
        ProfileCommand::create_command().into(),
        KickCommand::create_command().into(),
    ];

    let client = state.http().interaction(application_id);

    if let Err(error) = client.set_global_commands(&commands).exec().await {
        error!(error = ?error, "failed to register commands");
    }
}
