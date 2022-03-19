//! Interactions responses.
//!
//! This type hold types and traits used to respond to an interaction.
//!

use std::error::Error;

use raidprotect_model::ClusterState;
use tracing::error;
use twilight_model::{
    application::interaction::ApplicationCommand,
    channel::{embed::Embed, message::MessageFlags},
    http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    id::{
        marker::{ApplicationMarker, InteractionMarker},
        Id,
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::embed;

/// Credentials used to respond to an interaction.
#[derive(Debug)]
pub struct CommandResponder {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// ID of the associated application.
    pub application_id: Id<ApplicationMarker>,
    /// Token of the command.
    pub token: String,
}

impl CommandResponder {
    /// Initialize a new [`CommandResponder`] from an incoming command data.
    pub fn from_command(command: &ApplicationCommand) -> Self {
        Self {
            id: command.id,
            application_id: command.application_id,
            token: command.token.clone(),
        }
    }

    /// Send a response to an interaction.
    pub async fn respond(&self, state: &ClusterState, response: InteractionResponseData) {
        let client = state.http().interaction(self.application_id);
        let response = InteractionResponse {
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

/// Convert a type into [`InteractionResponseData`].
///
/// This type is used for interaction responses. It is implemented for common
/// types such as [`Embed`].
pub trait IntoResponse {
    /// Convert this type into [`InteractionResponseData`].
    fn into_response(self) -> InteractionResponseData;
}

impl<E> IntoResponse for Result<CommandResponse, E>
where
    E: InteractionError,
{
    fn into_response(self) -> InteractionResponseData {
        let response = match self {
            Ok(response) => response,
            Err(error) => match error.into_error() {
                InteractionErrorKind::Response(response) => *response,
                InteractionErrorKind::Internal(error) => {
                    tracing::error!(error = %error, "error occurred when processing interaction `{}`", E::INTERACTION_NAME);

                    embed::error::internal_error()
                }
            },
        };

        response.into_response()
    }
}

/// Response to a bot command.
///
/// This enum contains types that can be used to respond to a bot command. The
/// [`CommandResponse::Custom`] variant can be used to respond with a custom
/// [`InteractionResponseData`].
#[derive(Debug, Clone)]
pub enum CommandResponse {
    /// Respond with an embed.
    Embed(Embed),
    /// Respond with an embed sent as ephemeral message.
    EphemeralEmbed(Embed),
    /// Respond with a custom [`InteractionResponseData`].
    Custom(InteractionResponseData),
}

impl IntoResponse for CommandResponse {
    fn into_response(self) -> InteractionResponseData {
        match self {
            CommandResponse::Embed(embed) => InteractionResponseDataBuilder::new()
                .embeds([embed])
                .build(),
            CommandResponse::EphemeralEmbed(embed) => InteractionResponseDataBuilder::new()
                .embeds([embed])
                .flags(MessageFlags::EPHEMERAL)
                .build(),
            CommandResponse::Custom(response) => response,
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
    Response(Box<CommandResponse>),
    Internal(Box<dyn Error + Send + Sync>),
}

impl InteractionErrorKind {
    /// Initialize a new [`InteractionErrorKind::Response`].
    pub fn response(response: CommandResponse) -> Self {
        Self::Response(Box::new(response))
    }

    /// Initialize a new [`InteractionErrorKind::Internal`].
    pub fn internal(error: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal(Box::new(error))
    }
}
