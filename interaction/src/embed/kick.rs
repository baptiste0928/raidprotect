//! Embed for the kick command.

use raidprotect_translations::Lang;
use raidprotect_util::text::TextProcessExt;
use twilight_util::builder::embed::EmbedBuilder;

use super::COLOR_RED;
use crate::response::InteractionResponse;

/// User is not a server member.
pub fn not_member(user: String) -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(Lang::Fr.kick_not_member(user.remove_markdown().truncate(30)))
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Author is missing the `KICK_MEMBERS` permission
pub fn missing_permission() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(Lang::Fr.kick_missing_permission_title())
        .description(Lang::Fr.kick_missing_permission_description())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Bot is missing the `KICK_MEMBERS` permission
pub fn bot_missing_permission() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(Lang::Fr.kick_bot_missing_permission_title())
        .description(Lang::Fr.bot_missing_permission())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// User cannot kick due to the role hierarchy
pub fn user_hierarchy() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(Lang::Fr.kick_missing_permission_title())
        .description(Lang::Fr.hierarchy_user())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Bot cannot kick due to the role hierarchy
pub fn bot_hierarchy() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(Lang::Fr.kick_bot_missing_permission_title())
        .description(Lang::Fr.hierarchy_bot())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

/// Member is the guild owner, and thus cannot be kicked
pub fn member_owner() -> InteractionResponse {
    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .title(Lang::Fr.kick_missing_permission_title())
        .description(Lang::Fr.hierarchy_owner())
        .build();

    InteractionResponse::EphemeralEmbed(embed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_member() {
        not_member("test".to_string());
    }

    #[test]
    fn test_missing_permission() {
        missing_permission();
    }

    #[test]
    fn test_bot_missing_permission() {
        bot_missing_permission();
    }

    #[test]
    fn test_user_hierarchy() {
        user_hierarchy();
    }

    #[test]
    fn test_bot_hierarchy() {
        bot_hierarchy();
    }

    #[test]
    fn test_member_owner() {
        member_owner();
    }
}
