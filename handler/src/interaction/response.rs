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

impl IntoResponse for InteractionResponseData {
    fn into_response(self) -> InteractionResponseData {
        self
    }
}

impl IntoResponse for Embed {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseDataBuilder::new().embeds([self]).build()
    }
}

impl<T: IntoResponse, E: InteractionError> IntoResponse for Result<T, E> {
    fn into_response(self) -> InteractionResponseData {
        match self {
            Ok(value) => value.into_response(),
            Err(error) => error.into_error().into_response(),
        }
    }
}

/// Embed that respond using an ephemeral interaction callback.
///
/// This type wraps an [`Embed`] and implement [`IntoResponse`]. It can be
/// easily converted to and from an embed using the [`From`] trait.
#[derive(Debug, Clone)]
pub struct EphemeralEmbed(pub Embed);

impl IntoResponse for EphemeralEmbed {
    fn into_response(self) -> InteractionResponseData {
        InteractionResponseDataBuilder::new()
            .embeds([self.0])
            .flags(MessageFlags::EPHEMERAL)
            .build()
    }
}

impl From<Embed> for EphemeralEmbed {
    fn from(embed: Embed) -> Self {
        EphemeralEmbed(embed)
    }
}

impl From<EphemeralEmbed> for Embed {
    fn from(embed: EphemeralEmbed) -> Self {
        embed.0
    }
}

/// Error returned by interactions.
///
/// This trait represent an error returned by an interaction. Two kind of errors
/// are possible:
/// - [`InteractionErrorData::Callback`] represent a recoverable error that
///   display a custom user-friendly error message.
/// - [`InteractionErrorData::Internal`] represent a non-recoverable internal
///   error that display a generic error message and is logged.
pub trait InteractionError {
    /// Convert this type into [`InteractionErrorData`].
    fn into_error(self) -> InteractionErrorData;
}

/// Interaction error data.
///
/// See the [`InteractionError`] trait for more information.
#[derive(Debug)]
pub enum InteractionErrorData {
    Callback(Box<InteractionResponseData>),
    Internal {
        name: String,
        error: Box<dyn Error + Send + Sync>,
    },
}

impl InteractionErrorData {
    /// Initialize a new [`InteractionErrorData::Callback`].
    pub fn callback(callback: impl IntoResponse) -> Self {
        Self::Callback(Box::new(callback.into_response()))
    }

    /// Initialize a new [`InteractionErrorData::Internal`].
    pub fn internal(name: Option<&str>, error: impl Error + Send + Sync + 'static) -> Self {
        Self::Internal {
            name: name.unwrap_or("unknown").into(),
            error: Box::new(error),
        }
    }
}

impl IntoResponse for InteractionErrorData {
    fn into_response(self) -> InteractionResponseData {
        match self {
            InteractionErrorData::Callback(callback) => *callback,
            InteractionErrorData::Internal { name, error } => {
                tracing::error!(error = %error, "error occurred when processing interaction {}", name);

                embed::error::internal_error().into_response()
            }
        }
    }
}
