use std::sync::Arc;

use anyhow::bail;
use rosetta_i18n::Language;
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
    command::{help::HelpCommand, moderation::KickCommand, profile::ProfileCommand},
    component::PostInChat,
    embed,
    response::{InteractionResponder, InteractionResponse},
    util::InteractionExt,
};
use crate::{cluster::ClusterState, translations::Lang};

/// Handle incoming [`Interaction`].
pub async fn handle_interaction(interaction: Interaction, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_interaction(&interaction);
    debug!(id = ?interaction.id, "received {} interaction", interaction.kind.kind());

    let lang = interaction.locale().unwrap_or_else(|_| Lang::fallback());

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
                .respond(&state, embed::error::internal_error(lang))
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
        "help" => HelpCommand::handle(interaction, state).await,
        name => {
            warn!(name = name, "received unknown command");

            Ok(embed::error::unknown_command(interaction.locale()?))
        }
    }
}

/// Handle incoming component interaction
async fn handle_component(
    interaction: Interaction,
    state: &ClusterState,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::MessageComponent(data)) => data.custom_id.clone(),
        _ => bail!("expected message component data"),
    };

    let (name, component_id) = match custom_id.split_once(':') {
        Some((name, id)) => (name, id),
        None => bail!("expected custom_id to contain ':'"),
    };

    match name {
        "post-in-chat" => PostInChat::handle(interaction, component_id, state).await,
        name => {
            warn!(name = name, "received unknown component");

            Ok(embed::error::unknown_command(interaction.locale()?))
        }
    }
}

/// Handle incoming modal interaction
async fn handle_modal(
    interaction: Interaction,
    _state: &ClusterState,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::ModalSubmit(data)) => &*data.custom_id,
        _ => bail!("expected modal submit data"),
    };

    let (name, _modal_id) = match custom_id.split_once(':') {
        Some((name, id)) => (name, id),
        None => bail!("expected custom_id to contain ':'"),
    };

    match name {
        "sanction" => bail!("not implemented"),
        name => {
            warn!(name = name, "received unknown modal");

            Ok(embed::error::unknown_command(interaction.locale()?))
        }
    }
}

/// Register commands to the Discord API.
pub async fn register_commands(state: &ClusterState, application_id: Id<ApplicationMarker>) {
    let commands: Vec<Command> = vec![
        ProfileCommand::create_command().into(),
        KickCommand::create_command().into(),
        HelpCommand::create_command().into(),
    ];

    let client = state.http().interaction(application_id);

    if let Err(error) = client.set_global_commands(&commands).exec().await {
        error!(error = ?error, "failed to register commands");
    }
}
