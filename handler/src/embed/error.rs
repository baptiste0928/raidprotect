//! Error embeds.

use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder};

use crate::{interaction::response::CommandResponse, translations::Lang};

use super::COLOR_RED;

/// Internal error embed
pub fn internal_error() -> CommandResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.internal_error_title())
        .color(COLOR_RED)
        .description(Lang::Fr.internal_error_description())
        .footer(EmbedFooterBuilder::new(
            "Okay, Houston, I believe we've had a problem here ...", // No translation here
        ))
        .build()
        .unwrap();

    CommandResponse::EphemeralEmbed(embed)
}

/// Unknown command received
pub fn unknown_command() -> CommandResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.unknown_command_title())
        .color(COLOR_RED)
        .description(Lang::Fr.unknown_command_description())
        .build()
        .unwrap();

    CommandResponse::Embed(embed)
}

/// Command not available in direct messages
pub fn guild_only() -> CommandResponse {
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.guild_only_title())
        .color(COLOR_RED)
        .description(Lang::Fr.guild_only_description())
        .build()
        .unwrap();

    CommandResponse::Embed(embed)
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
    fn test_guild_only() {
        guild_only();
    }
}
