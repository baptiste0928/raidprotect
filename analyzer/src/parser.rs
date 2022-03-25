//! Message parser.

use any_ascii::any_ascii;
use linkify::{LinkFinder, LinkKind};
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

/// Parsed message content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedMessage {
    /// Original message content.
    pub original: String,
    /// List of message words.
    ///
    /// The words are splitted according to the Unicode specification and each
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
    User(Id<UserMarker>),
    Role(Id<RoleMarker>),
}

/// Kind of message mention. This type is used when parsing [`MessageMention`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MentionKind {
    User,
    Role,
}

impl ParsedMessage {
    pub fn parse(message: String) -> Self {
        let words = message.unicode_words().map(any_ascii).collect();
        let links = LinkFinder::new()
            .kinds(&[LinkKind::Email])
            .links(&message)
            .filter_map(|link| MessageLink::parse(link.as_str()))
            .collect();

        Self {
            original: message,
            words,
            links,
            mentions: todo!(),
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
    /// Parse a single mention into [`MessageMention`].
    fn parse(mention: &str) -> Option<Self> {
        let mut chars = mention.chars();
        let mut id = String::new();

        // Id must start with '<'
        match chars.next()? {
            '<' => {}
            _ => return None,
        }

        // Parse second character, either '&' for role or '!' (or nothing) for
        // user identifier.
        let char = chars.next()?;
        let kind = match char {
            '&' => MentionKind::Role,
            '!' => MentionKind::User,
            '0'..='9' => {
                id.push(char);

                MentionKind::User
            }
            _ => return None,
        };

        // Parse id characters
        loop {
            let char = chars.next()?;
            match char {
                '0'..='9' => id.push(char),
                '>' => break,
                _ => return None,
            }
        }

        // Parse id as number
        let id: u64 = id.parse().ok()?;

        match kind {
            MentionKind::User => Some(MessageMention::User(Id::new_checked(id)?)),
            MentionKind::Role => Some(MessageMention::Role(Id::new_checked(id)?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_invite() {
        assert!(matches!(
            MessageLink::parse("https://discord.gg/raidprotect"),
            Some(MessageLink::Invite(_))
        ));

        assert!(matches!(
            MessageLink::parse("https://discord.com/invite/raidprotect"),
            Some(MessageLink::Invite(_))
        ));

        assert!(matches!(
            MessageLink::parse("https://discordapp.com/invite/raidprotect"),
            Some(MessageLink::Invite(_))
        ));
    }

    #[test]
    fn test_link_media() {
        assert!(matches!(
            MessageLink::parse("https://cdn.discordapp.com/attachments/618052865725825044/956984958184984586/Capture_decran_2022-03-25_193605.png"),
            Some(MessageLink::Media(_))
        ));

        assert!(matches!(
            MessageLink::parse("https://cdn.discordapp.com/attachments/796185053351772191/872796992357695548/video0-16-2.mp4"),
            Some(MessageLink::Media(_))
        ));
    }

    #[test]
    fn test_link_other() {
        assert!(matches!(
            MessageLink::parse("https://raidprotect.org/"),
            Some(MessageLink::Other(_))
        ));
    }
}
