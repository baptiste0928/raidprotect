//! Embeds for the captcha configuration commands.
use twilight_util::builder::embed::EmbedBuilder;

use super::COLOR_RED;
use crate::{interaction::response::InteractionResponse, translations::Lang};

/// Captcha not enabled on the server.
pub fn not_enabled(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_not_enabled_title())
        .description(lang.captcha_not_enabled_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}
