//! Embed for the kick command.

use twilight_util::builder::embed::EmbedBuilder;

use super::COLOR_RED;
use crate::{interaction::response::InteractionResponse, translations::Lang, util::TextProcessExt};

/// User is not a server member.
pub fn not_member(user: String, lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.kick_not_member(user.remove_markdown().truncate(30)))
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Bot is missing the `KICK_MEMBERS` permission
pub fn bot_missing_permission(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.kick_bot_missing_permission_title())
        .description(lang.bot_missing_permission())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// User cannot kick due to the role hierarchy
pub fn user_hierarchy(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.kick_missing_permission_title())
        .description(lang.hierarchy_user())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Bot cannot kick due to the role hierarchy
pub fn bot_hierarchy(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.kick_bot_missing_permission_title())
        .description(lang.hierarchy_bot())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Member is the guild owner, and thus cannot be kicked
pub fn member_owner(lang: Lang) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(lang.kick_missing_permission_title())
        .description(lang.hierarchy_owner())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_member() {
        not_member("test".to_string(), Lang::DEFAULT);
    }

    #[test]
    fn test_bot_missing_permission() {
        bot_missing_permission(Lang::DEFAULT);
    }

    #[test]
    fn test_user_hierarchy() {
        user_hierarchy(Lang::DEFAULT);
    }

    #[test]
    fn test_bot_hierarchy() {
        bot_hierarchy(Lang::DEFAULT);
    }

    #[test]
    fn test_member_owner() {
        member_owner(Lang::DEFAULT);
    }
}
