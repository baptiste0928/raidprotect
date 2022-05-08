//! Interactions responses.
//!
//! This module exports types and traits used to respond to an interaction.

use std::error::Error;

use raidprotect_state::ClusterState;
use tracing::error;
use twilight_http::error::ErrorType;
use twilight_model::{
    application::{
        component::Component,
        interaction::{ApplicationCommand, MessageComponentInteraction},
    },
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

use crate::embed;
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
    /// Initialize a new [`InteractionResponder`] from an incoming command data.
    pub fn from_command(command: &ApplicationCommand) -> Self {
        Self {
            id: command.id,
            application_id: command.application_id,
            token: command.token.clone(),
        }
    }

    /// Initialize a new [`InteractionResponder`] from an incoming component data.
    pub fn from_component(component: &MessageComponentInteraction) -> Self {
        Self {
            id: component.id,
            application_id: component.application_id,
            token: component.token.clone(),
        }
    }

    /// Send a response to an interaction.
    pub async fn respond(&self, state: &ClusterState, response: HttpInteractionResponse) {
        let client = state.http().interaction(self.application_id);

        if let Err(error) = client
            .create_response(self.id, &self.token, &response)
            .exec()
            .await
        {
            let body = match error.kind() {
                ErrorType::Response { body, .. } => std::str::from_utf8(body).ok(),
                _ => None,
            };

            error!(error = %error, body = ?body, "failed to respond to interaction");
        }
    }
}

/// Response to an interaction.
///
/// This enum contains types that can be used to respond to an interaction. The
/// [`InteractionResponse::Custom`] variant can be used to respond with a custom
/// [`InteractionResponseData`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InteractionResponse {
    /// Respond with an embed.
    Embed(Embed),
    /// Respond with an embed sent as ephemeral message.
    EphemeralEmbed(Embed),
    /// Respond with a modal.
    Modal {
        custom_id: String,
        title: String,
        components: Component,
    },
    /// Respond with a custom [`InteractionResponseData`].
    Custom(InteractionResponseData),
}

/// Convert a type into [`InteractionResponseData`]..
pub trait IntoResponse {
    /// Convert this type into [`InteractionResponseData`].
    fn into_response(self) -> HttpInteractionResponse;
}

impl IntoResponse for InteractionResponse {
    fn into_response(self) -> HttpInteractionResponse {
        let kind = match self {
            Self::Modal { .. } => InteractionResponseType::Modal,
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
                    .components([components])
                    .build(),
            ),
            Self::Custom(response) => Some(response),
        };

        HttpInteractionResponse { kind, data }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: InteractionError,
{
    fn into_response(self) -> HttpInteractionResponse {
        match self {
            Ok(response) => response.into_response(),
            Err(error) => match error.into_error() {
                InteractionErrorKind::Response(response) => response.into_response(),
                InteractionErrorKind::Internal(error) => {
                    tracing::error!(error = %error, "error occurred when processing interaction `{}`", E::INTERACTION_NAME);

                    embed::error::internal_error().into_response()
                }
            },
        }
    }
}

/// Error returned by interactions.
///
/// This trait represent an error returned by an interaction. Two kind of errors
/// are possible:
/// - [`InteractionErrorKind::Response`] represent a recoverable error that
///   display a custom user-friendly error message.
/// - [`InteractionErrorKind::Internal`] represent a non-recoverable internal
///   error that display a generic error message and is logged.
pub trait InteractionError {
    /// Name of the interaction.
    const INTERACTION_NAME: &'static str;

    /// Convert this type into [`InteractionErrorKind`].
    fn into_error(self) -> InteractionErrorKind;
}

/// Interaction error data.
///
/// See the [`InteractionError`] trait for more information.
#[derive(Debug)]
pub enum InteractionErrorKind {
    Response(Box<InteractionResponse>),
    Internal(Box<dyn Error + Send + Sync>),
}

impl InteractionErrorKind {
    /// Initialize a new [`InteractionErrorKind::Response`].
    pub fn response(response: InteractionResponse) -> Self {
        Self::Response(Box::new(response))
    }

    /// Initialize a new [`InteractionErrorKind::Internal`].
    pub fn internal(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(error))
    }
}

impl From<InteractionResponse> for InteractionErrorKind {
    fn from(response: InteractionResponse) -> Self {
        Self::response(response)
    }
}
