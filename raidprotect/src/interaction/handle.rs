use std::{str::FromStr, sync::Arc};

use anyhow::bail;
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
    command::{
        config::ConfigCommand, help::HelpCommand, moderation::KickCommand, profile::ProfileCommand,
    },
    component::{
        captcha::{CaptchaDisable, CaptchaEnable},
        PostInChat,
    },
    embed,
    response::{InteractionResponder, InteractionResponse},
    util::{CustomId, InteractionExt},
};
use crate::{cluster::ClusterState, translations::Lang};

/// Handle incoming [`Interaction`].
pub async fn handle_interaction(interaction: Interaction, state: Arc<ClusterState>) {
    let responder = InteractionResponder::from_interaction(&interaction);
    debug!(id = ?interaction.id, "received {} interaction", interaction.kind.kind());

    let lang = interaction.locale().unwrap_or(Lang::DEFAULT);

    let response = match interaction.kind {
        InteractionType::ApplicationCommand => handle_command(interaction, state.clone()).await,
        InteractionType::MessageComponent => handle_component(interaction, state.clone()).await,
        InteractionType::ModalSubmit => handle_modal(interaction, state.clone()).await,
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
    state: Arc<ClusterState>,
) -> Result<InteractionResponse, anyhow::Error> {
    let name = match &interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => &*data.name,
        _ => bail!("expected application command data"),
    };

    match name {
        "config" => ConfigCommand::handle(interaction, &state).await,
        "help" => HelpCommand::handle(interaction, &state).await,
        "kick" => KickCommand::handle(interaction, &state).await,
        "profile" => ProfileCommand::handle(interaction, &state).await,
        name => {
            warn!(name = name, "received unknown command");

            Ok(embed::error::unknown_command(interaction.locale()?))
        }
    }
}

/// Handle incoming component interaction
async fn handle_component(
    interaction: Interaction,
    state: Arc<ClusterState>,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::MessageComponent(data)) => CustomId::from_str(&*data.custom_id)?,
        _ => bail!("expected message component data"),
    };

    match &*custom_id.name {
        "post-in-chat" => PostInChat::handle(interaction, custom_id, &state).await,
        "captcha-enable" => CaptchaEnable::handle(interaction, state).await,
        "captcha-disable" => CaptchaDisable::handle(interaction, state).await,
        name => {
            warn!(name = name, "received unknown component");

            Ok(embed::error::unknown_command(interaction.locale()?))
        }
    }
}

/// Handle incoming modal interaction
async fn handle_modal(
    interaction: Interaction,
    _state: Arc<ClusterState>,
) -> Result<InteractionResponse, anyhow::Error> {
    let custom_id = match &interaction.data {
        Some(InteractionData::ModalSubmit(data)) => CustomId::from_str(&*data.custom_id)?,
        _ => bail!("expected modal submit data"),
    };

    match &*custom_id.name {
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
        ConfigCommand::create_command().into(),
        HelpCommand::create_command().into(),
        KickCommand::create_command().into(),
        ProfileCommand::create_command().into(),
    ];

    let client = state.http().interaction(application_id);

    if let Err(error) = client.set_global_commands(&commands).exec().await {
        error!(error = ?error, "failed to register commands");
    }
}
