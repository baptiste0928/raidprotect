//! Message parser.

use any_ascii::any_ascii;
use lazy_static::lazy_static;
use linkify::{LinkFinder, LinkKind};
use regex::Regex;
use twilight_model::id::{
    marker::{RoleMarker, UserMarker},
    Id,
};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

/// Domains used for Discord invitations link.
const INVITE_DOMAINS: [&str; 3] = ["discord.gg", "discord.com", "discordapp.com"];

/// Extensions of media files.
const MEDIA_EXT: [&str; 9] = [
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".webm", ".mp4", ".avi", ".mov",
];

lazy_static! {
    // Regex from discord.js, improved to ensure id does not start with '0'?=.
    //
    // https://github.com/discordjs/discord.js/blob/988a51b7641f8b33cc9387664605ddc02134859d/src/structures/MessageMentions.js#L215

    /// Regex that matches user mentions (like <@80351110224678912>)
    static ref USER_MENTION: Regex = Regex::new(r"<@!?([1-9][0-9]{16,18})>").unwrap();

    /// Regex that matches role mentions (like <@&165511591545143296>)
    static ref ROLE_MENTION: Regex = Regex::new(r"<@&([1-9][0-9]{16,18})>").unwrap();

    /// Regex that matches @everyone and @here mentions
    static ref EVERYONE_MENTION: Regex = Regex::new(r"@(everyone|here)").unwrap();
}

/// Parsed message content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    /// Original message content.
    pub original: String,
    /// List of message words.
    ///
    /// The words are split according to the Unicode specification and each
    /// character is converted into ASCII.
    pub words: Vec<String>,
    /// List of message links.
    pub links: Vec<MessageLink>,
    /// List of message mentions.
    pub mentions: Vec<MessageMention>,
}

/// Kind of message link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageLink {
    Invite(Url),
    Media(Url),
    Other(Url),
}

/// Kind of message mention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageMention {
    /// User mention like `<@80351110224678912>`
    User(Id<UserMarker>),
    /// Role mention like `<@&165511591545143296>`
    Role(Id<RoleMarker>),
    /// `@everyone` or `@here` mention
    Everyone,
}

impl ParsedMessage {
    /// Parse message into [`ParsedMessage`]
    pub fn parse(message: String) -> Self {
        let words = message.unicode_words().map(any_ascii).collect();
        let mentions = MessageMention::match_mentions(&message);
        let links = LinkFinder::new()
            .kinds(&[LinkKind::Url])
            .links(&message)
            .filter_map(|link| MessageLink::parse(link.as_str()))
            .collect();

        Self {
            original: message,
            words,
            links,
            mentions,
        }
    }
}

impl MessageLink {
    /// Parse a single link into [`MessageLink`].
    fn parse(link: &str) -> Option<Self> {
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
}

impl MessageMention {
    /// Match all mentions from a message.
    fn match_mentions(message: &str) -> Vec<Self> {
        let mut mentions = Vec::new();

        // Early-return if the character '@' is not in the message
        if !message.contains('@') {
            return mentions;
        }

        // Capture user mentions
        for capture in USER_MENTION.captures_iter(message) {
            if let Ok(user_id) = capture[1].parse() {
                mentions.push(Self::User(Id::new(user_id)));
            }
        }

        // Capture role mentions
        for capture in ROLE_MENTION.captures_iter(message) {
            if let Ok(role_id) = capture[1].parse() {
                mentions.push(Self::Role(Id::new(role_id)));
            }
        }

        // Capture everyone mentions
        for _ in EVERYONE_MENTION.find_iter(message) {
            mentions.push(Self::Everyone);
        }

        mentions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_parsing() {
        assert_eq!(
            ParsedMessage::parse("Hello, world!".into()),
            ParsedMessage {
                original: "Hello, world!".into(),
                words: vec!["Hello".into(), "world".into()],
                links: Vec::new(),
                mentions: Vec::new(),
            }
        )
    }

    #[test]
    fn test_message_parsing_empty() {
        assert_eq!(
            ParsedMessage::parse("".into()),
            ParsedMessage {
                original: "".into(),
                words: Vec::new(),
                links: Vec::new(),
                mentions: Vec::new(),
            }
        )
    }

    #[test]
    fn test_message_parsing_unicode() {
        assert_eq!(
            ParsedMessage::parse("𝓗𝓮𝓵𝓵𝓸, 𝔀𝓸𝓻𝓵𝓭!".into()),
            ParsedMessage {
                original: "𝓗𝓮𝓵𝓵𝓸, 𝔀𝓸𝓻𝓵𝓭!".into(),
                words: vec!["Hello".into(), "world".into()],
                links: Vec::new(),
                mentions: Vec::new(),
            }
        )
    }

    #[test]
    fn test_message_parsing_link() {
        assert_eq!(
            ParsedMessage::parse("Join my server: https://discord.gg/raidprotect".into()),
            ParsedMessage {
                original: "Join my server: https://discord.gg/raidprotect".into(),
                words: vec![
                    "Join".into(),
                    "my".into(),
                    "server".into(),
                    "https".into(),
                    "discord.gg".into(),
                    "raidprotect".into()
                ],
                links: vec![MessageLink::Invite(
                    Url::parse("https://discord.gg/raidprotect").unwrap()
                )],
                mentions: Vec::new(),
            }
        )
    }

    #[test]
    fn test_message_parsing_mention() {
        assert_eq!(
            ParsedMessage::parse("Hello <@466578580449525760>".into()),
            ParsedMessage {
                original: "Hello <@466578580449525760>".into(),
                words: vec!["Hello".into(), "466578580449525760".into()],
                links: Vec::new(),
                mentions: vec![MessageMention::User(Id::new(466578580449525760))],
            }
        )
    }

    #[test]
    fn test_link_invite() {
        assert_eq!(
            MessageLink::parse("https://discord.gg/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discord.gg/raidprotect").unwrap()
            ))
        );

        assert_eq!(
            MessageLink::parse("https://discord.com/invite/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discord.com/invite/raidprotect").unwrap()
            ))
        );

        assert_eq!(
            MessageLink::parse("https://discordapp.com/invite/raidprotect"),
            Some(MessageLink::Invite(
                Url::parse("https://discordapp.com/invite/raidprotect").unwrap()
            ))
        );
    }

    #[test]
    fn test_link_media() {
        assert_eq!(
            MessageLink::parse("https://cdn.discordapp.com/attachments/618052865725825044/956984958184984586/Capture_decran_2022-03-25_193605.png"),
            Some(MessageLink::Media(Url::parse("https://cdn.discordapp.com/attachments/618052865725825044/956984958184984586/Capture_decran_2022-03-25_193605.png").unwrap()))
        );

        assert_eq!(
            MessageLink::parse("https://cdn.discordapp.com/attachments/796185053351772191/872796992357695548/video0-16-2.mp4"),
            Some(MessageLink::Media(Url::parse("https://cdn.discordapp.com/attachments/796185053351772191/872796992357695548/video0-16-2.mp4").unwrap()))
        );
    }

    #[test]
    fn test_link_other() {
        assert_eq!(
            MessageLink::parse("https://raidprotect.org/"),
            Some(MessageLink::Other(
                Url::parse("https://raidprotect.org/").unwrap()
            ))
        );
    }

    #[test]
    fn test_mention() {
        assert_eq!(
            MessageMention::match_mentions(
                "<@207852811596201985> <@!466578580449525760> <@&490525831806976011> @everyone"
            ),
            vec![
                MessageMention::User(Id::new(207852811596201985)),
                MessageMention::User(Id::new(466578580449525760)),
                MessageMention::Role(Id::new(490525831806976011)),
                MessageMention::Everyone,
            ]
        );
    }

    #[test]
    fn test_mention_fails() {
        assert_eq!(
            MessageMention::match_mentions("<@00000000000000000>"),
            Vec::new()
        );

        assert_eq!(
            MessageMention::match_mentions("<@&00000000000000000>"),
            Vec::new()
        );

        assert_eq!(MessageMention::match_mentions("<@!123>"), Vec::new());

        assert_eq!(
            MessageMention::match_mentions("@207852811596201985"),
            Vec::new()
        );
    }
}
