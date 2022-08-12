//! Error embeds.

use twilight_util::builder::embed::{EmbedBuilder, EmbedFooterBuilder};

use super::COLOR_RED;
use crate::{interaction::response::InteractionResponse, translations::Lang};

/// Internal error embed
pub fn internal_error(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(lang.internal_error_title())
        .color(COLOR_RED)
        .description(lang.internal_error_description())
        .footer(EmbedFooterBuilder::new(
            "Okay, Houston, I believe we've had a problem here ...", // No translation here
        ))
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Unknown command received
pub fn unknown_command(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(lang.unknown_command_title())
        .color(COLOR_RED)
        .description(lang.unknown_command_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

pub fn expired_interaction(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(lang.expired_interaction_title())
        .color(COLOR_RED)
        .description(lang.expired_interaction_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error() {
        internal_error(Lang::DEFAULT);
    }

    #[test]
    fn test_unknown_command() {
        unknown_command(Lang::DEFAULT);
    }

    #[test]
    fn test_expired_component() {
        expired_interaction(Lang::DEFAULT);
    }
}
