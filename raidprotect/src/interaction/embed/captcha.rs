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

/// Captcha already enabled on the server.
pub fn already_enabled(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_already_enabled_title())
        .description(lang.captcha_already_enabled_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Missing permission to send message in the logs channel.
pub fn missing_logs_permission(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_missing_logs_permission_title())
        .description(lang.bot_missing_permission())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Missing permission to give a role to new members.
pub fn missing_role_permission(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_missing_role_permission_title())
        .description(lang.bot_missing_permission())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Missing permissions to enable the captcha.
pub fn missing_enable_permission(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_missing_enable_permission_title())
        .description(lang.bot_missing_permission())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Missing permission to give a role due to the role hierarchy.
pub fn role_hierarchy(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.captcha_missing_role_permission_title())
        .description(lang.hierarchy_bot_role())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Role already configured as a verified role.
pub fn role_already_added(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_role_already_added())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Too many roles configured as verified roles.
pub fn role_too_many(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_role_too_many())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Role not configured as a verified role.
pub fn role_not_configured(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_role_not_configured())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Error while creating the captcha role.
pub fn role_error(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_role_error())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Error while creating the captcha channel.
pub fn channel_error(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_channel_error())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Captcha has been regenerated too many times.
pub fn regenerate_error(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(lang.captcha_error_title())
        .color(COLOR_RED)
        .description(lang.captcha_regenerate_error_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Error because no captcha was found.
pub fn captcha_not_found(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .title(lang.captcha_error_title())
        .color(COLOR_RED)
        .description(lang.captcha_not_found_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}
