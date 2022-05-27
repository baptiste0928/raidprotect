//! Message parser.
//!
//! This module is used to convert an incoming [`Message`] into a parsed
//! [`CachedMessage`].

use any_ascii::any_ascii;
use linkify::{LinkFinder, LinkKind};
use raidprotect_model::cache::model::message::{CachedMessage, MessageLink};
use twilight_model::channel::Message;
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use super::ALLOWED_MESSAGES_TYPES;

/// Domains used for Discord invitations link.
const INVITE_DOMAINS: [&str; 3] = ["discord.gg", "discord.com", "discordapp.com"];

/// Extensions of media files.
const MEDIA_EXT: [&str; 9] = [
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".webm", ".mp4", ".avi", ".mov",
];

/// Parse incoming [`Message`] into a [`CachedMessage`].
pub fn parse_message(message: &Message) -> CachedMessage {
    // Only these message types are processed.
    // This must be enforced in the gateway crate.
    debug_assert!(
        message.guild_id.is_some(),
        "message wasn't sent from a guild"
    );
    debug_assert!(
        ALLOWED_MESSAGES_TYPES.contains(&message.kind),
        "unsupported message type"
    );

    let words = message.content.unicode_words().map(any_ascii).collect();
    let mention_users = message.mentions.iter().map(|mention| mention.id).collect();
    let links = LinkFinder::new()
        .kinds(&[LinkKind::Url])
        .links(&message.content)
        .filter_map(|link| parse_link(link.as_str()))
        .collect();

    CachedMessage {
        id: message.id,
        author_id: message.author.id,
        channel_id: message.channel_id,
        content: message.content.clone(),
        timestamp: message.timestamp,
        words,
        attachments: message.attachments.clone(),
        links,
        mention_everyone: message.mention_everyone,
        mention_users,
        mention_roles: message.mention_roles.clone(),
    }
}

fn parse_link(link: &str) -> Option<MessageLink> {
    let url = Url::parse(link).ok()?;

    if INVITE_DOMAINS.contains(&url.domain()?) {
        return Some(MessageLink::Invite(url));
    }

    let last_path = url.path_segments()?.last()?;
    for extension in MEDIA_EXT {
        if last_path.ends_with(extension) {
            return Some(MessageLink::Media(url));
        }
    }

    Some(MessageLink::Other(url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_invite() {
        assert_eq!(
            parse_link("https://discord.gg/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discord.gg/raidprotect").unwrap()
            ))
        );

        assert_eq!(
            parse_link("https://discord.com/invite/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discord.com/invite/raidprotect").unwrap()
            ))
        );

        assert_eq!(
            parse_link("https://discordapp.com/invite/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discordapp.com/invite/raidprotect").unwrap()
            ))
        );
    }

    #[test]
    fn test_link_media() {
        assert_eq!(
            parse_link("https://cdn.discordapp.com/attachments/618052865725825044/956984958184984586/Capture_decran_2022-03-25_193605.png"),
            Some(MessageLink::Media(Url::parse("https://cdn.discordapp.com/attachments/618052865725825044/956984958184984586/Capture_decran_2022-03-25_193605.png").unwrap()))
        );

        assert_eq!(
            parse_link("https://cdn.discordapp.com/attachments/796185053351772191/872796992357695548/video0-16-2.mp4"),
            Some(MessageLink::Media(Url::parse("https://cdn.discordapp.com/attachments/796185053351772191/872796992357695548/video0-16-2.mp4").unwrap()))
        );
    }

    #[test]
    fn test_link_other() {
        assert_eq!(
            parse_link("https://raidprotect.org/"),
            Some(MessageLink::Other(
                Url::parse("https://raidprotect.org/").unwrap()
            ))
        );
    }
}
