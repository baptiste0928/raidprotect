//! Error embeds.

use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

use super::COLOR_RED;
use crate::{interaction::response::InteractionResponse, translations::Lang};

/// Internal error embed
pub fn internal_error() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.internal_error_title())
        .color(COLOR_RED)
        .description(Lang::Fr.internal_error_description())
        .footer(EmbedFooterBuilder::new(
            "Okay, Houston, I believe we've had a problem here ...", // No translation here
        ))
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Unknown command received
pub fn unknown_command() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.unknown_command_title())
        .color(COLOR_RED)
        .description(Lang::Fr.unknown_command_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

pub fn expired_interaction() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.expired_interaction_title())
        .color(COLOR_RED)
        .description(Lang::Fr.expired_interaction_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error() {
        internal_error();
    }

    #[test]
    fn test_unknown_command() {
        unknown_command();
    }

    #[test]
    fn test_expired_component() {
        expired_interaction();
    }
}
