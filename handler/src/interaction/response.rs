//! Interactions reponses.
//!
//! This type hold types and traits used to respond to an interaction.
//!

use std::error::Error;

use twilight_model::{
    application::callback::CallbackData,
    channel::{embed::Embed, message::MessageFlags},
};
use twilight_util::builder::CallbackDataBuilder;

use crate::embed;

/// Convert a type into [`CallbackData`].
///
/// This type is used for interaction responses. It is implemented for common
/// types such as [`Embed`].
pub trait IntoResponse {
    /// Convert this type into [`CallbackData`].
    fn into_callback(self) -> CallbackData;
}

impl IntoResponse for CallbackData {
    fn into_callback(self) -> CallbackData {
        self
    }
}

impl IntoResponse for Embed {
    fn into_callback(self) -> CallbackData {
        CallbackDataBuilder::new().embeds([self]).build()
    }
}

impl<T: IntoResponse, E: InteractionError> IntoResponse for Result<T, E> {
    fn into_callback(self) -> CallbackData {
        match self {
            Ok(value) => value.into_callback(),
            Err(error) => error.into_error().into_callback(),
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
    fn into_callback(self) -> CallbackData {
        CallbackDataBuilder::new()
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
    Callback(CallbackData),
    Internal {
        name: String,
        error: Box<dyn Error + Send + Sync>,
    },
}

impl InteractionErrorData {
    /// Initialize a new [`InteractionErrorData::Callback`].
    pub fn callback(callback: impl IntoResponse) -> Self {
        Self::Callback(callback.into_callback())
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
    fn into_callback(self) -> CallbackData {
        match self {
            InteractionErrorData::Callback(callback) => callback,
            InteractionErrorData::Internal { name, error } => {
                tracing::error!(error = %error, "error occurred when processing interaction {}", name);

                embed::error::internal_error().into_callback()
            }
        }
    }
}
