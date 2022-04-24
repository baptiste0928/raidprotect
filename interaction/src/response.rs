//! Interactions responses.
//!
//! This module exports types and traits used to respond to an interaction.

use std::error::Error;

use raidprotect_model::{interaction::InteractionResponse, ClusterState};
use tracing::error;
use twilight_model::{
    application::interaction::{ApplicationCommand, MessageComponentInteraction},
    channel::message::MessageFlags,
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
    pub async fn respond(&self, state: &ClusterState, response: InteractionResponseData) {
        let client = state.http().interaction(self.application_id);
        let response = HttpInteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        };

        if let Err(error) = client
            .create_response(self.id, &self.token, &response)
            .exec()
            .await
        {
            error!(error = %error, "failed to respond to interaction");
        }
    }
}

/// Convert a type into [`InteractionResponseData`]..
pub trait IntoResponse {
    /// Convert this type into [`InteractionResponseData`].
    fn into_response(self) -> InteractionResponseData;
}

impl IntoResponse for InteractionResponse {
    fn into_response(self) -> InteractionResponseData {
        match self {
            InteractionResponse::Embed(embed) => InteractionResponseDataBuilder::new()
                .embeds([embed])
                .build(),
            InteractionResponse::EphemeralEmbed(embed) => InteractionResponseDataBuilder::new()
                .embeds([embed])
                .flags(MessageFlags::EPHEMERAL)
                .build(),
            InteractionResponse::Custom(response) => response,
        }
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: InteractionError,
{
    fn into_response(self) -> InteractionResponseData {
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
