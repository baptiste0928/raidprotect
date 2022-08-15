//! Interactions responses.
//!
//! This module exports types and traits used to respond to an interaction.

use tracing::error;
use twilight_model::{
    application::{component::Component, interaction::Interaction},
    channel::{embed::Embed, message::MessageFlags},
    http::interaction::{
        InteractionResponse as HttpInteractionResponse, InteractionResponseData,
        InteractionResponseType,
    },
    id::{
        marker::{ApplicationMarker, InteractionMarker},
        Id,
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::cluster::ClusterState;

/// Credentials used to respond to an interaction.
#[derive(Debug)]
pub struct InteractionResponder {
    /// ID of the interaction.
    pub id: Id<InteractionMarker>,
    /// ID of the associated application.
    pub application_id: Id<ApplicationMarker>,
    /// Token of the command.
    pub token: String,
}

impl InteractionResponder {
    /// Initialize a new [`InteractionResponder`] from an incoming interaction.
    pub fn from_interaction(interaction: &Interaction) -> Self {
        Self {
            id: interaction.id,
            application_id: interaction.application_id,
            token: interaction.token.clone(),
        }
    }

    /// Send a response to an interaction.
    pub async fn respond(&self, state: &ClusterState, response: InteractionResponse) {
        let client = state.http().interaction(self.application_id);

        if let Err(error) = client
            .create_response(self.id, &self.token, &response.into_http())
            .exec()
            .await
        {
            error!(error = ?error, "failed to respond to interaction");
        }
    }
}

/// Response to an interaction.
///
/// This enum contains types that can be used to respond to an interaction.
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionResponse {
    /// Respond with an embed.
    #[allow(unused)]
    Embed(Embed),
    /// Respond with an embed sent as ephemeral message.
    EphemeralEmbed(Embed),
    /// Respond with a modal.
    Modal {
        custom_id: String,
        title: String,
        components: Vec<Component>,
    },
    /// Respond with an ephemeral [`DeferredChannelMessageWithSource`] interaction type.
    ///
    /// [`DeferredChannelMessageWithSource`]: InteractionResponseType::DeferredChannelMessageWithSource
    EphemeralDeferredMessage,
    /// Respond with a raw [`HttpInteractionResponse`].
    Raw {
        kind: InteractionResponseType,
        data: Option<InteractionResponseData>,
    },
}

impl InteractionResponse {
    /// Convert the response into a [`HttpInteractionResponse`].
    fn into_http(self) -> HttpInteractionResponse {
        let kind = match self {
            Self::Modal { .. } => InteractionResponseType::Modal,
            Self::EphemeralDeferredMessage => {
                InteractionResponseType::DeferredChannelMessageWithSource
            }
            Self::Raw { kind, .. } => kind,
            _ => InteractionResponseType::ChannelMessageWithSource,
        };

        let data = match self {
            Self::Embed(embed) => Some(
                InteractionResponseDataBuilder::new()
                    .embeds([embed])
                    .build(),
            ),
            Self::EphemeralEmbed(embed) => Some(
                InteractionResponseDataBuilder::new()
                    .embeds([embed])
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
            ),
            Self::Modal {
                custom_id,
                title,
                components,
            } => Some(
                InteractionResponseDataBuilder::new()
                    .custom_id(custom_id)
                    .title(title)
                    .components(components)
                    .build(),
            ),
            Self::EphemeralDeferredMessage => Some(
                InteractionResponseDataBuilder::new()
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
            ),
            Self::Raw { data, .. } => data,
        };

        HttpInteractionResponse { kind, data }
    }
}
