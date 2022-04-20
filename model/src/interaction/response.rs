//! Interaction response model.

use twilight_model::{channel::embed::Embed, http::interaction::InteractionResponseData};

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
    /// Respond with a custom [`InteractionResponseData`].
    Custom(InteractionResponseData),
}
