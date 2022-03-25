//! Message parser.

use any_ascii::any_ascii;
use twilight_model::id::{
    marker::{RoleMarker, UserMarker},
    Id,
};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

/// Domains used for Discord invitations link.
const INVITE_DOMAINS: [&str; 3] = ["discord.gg", "discord.com", "discordapp.com"];

/// Extensions of media files.
const MEDIA_EXT: [&str; 8] = [
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".webm", ".mp4", ".avi",
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

impl ParsedMessage {
    pub fn parse(message: String) -> Self {
        // Extract message words
        let words = message.unicode_words().map(any_ascii).collect();

        Self {
            original: message,
            words,
            links: todo!(),
            mentions: todo!(),
        }
    }
}

impl MessageLink {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_link() {
        assert!(matches!(
            MessageLink::parse("https://discord.gg/raidprotect"),
            Some(MessageLink::Invite(_))
        ));
    }
}
