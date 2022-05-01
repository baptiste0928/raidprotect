//! Embed for the kick command.

use raidprotect_model::interaction::InteractionResponse;
use raidprotect_translations::Lang;
use raidprotect_util::text::TextProcessExt;
use twilight_util::builder::embed::EmbedBuilder;

use super::COLOR_RED;

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
}
