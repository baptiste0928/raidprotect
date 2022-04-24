//! Utility functions to format Discord resources.

use std::fmt::{self, Display};

use twilight_model::{
    id::{marker::UserMarker, Id},
    user::User,
    util::ImageHash,
};

const USER_AVATAR_BASE: &str = "https://cdn.discordapp.com/avatars";
const DEFAULT_AVATAR_BASE: &str = "https://cdn.discordapp.com/embed/avatars";

/// Get a Discord user avatar link.
///
/// This method will return the default avatar if the user has no avatar.
pub fn avatar_url(user: &User, format: ImageFormat, size: ImageSize) -> String {
    match user.avatar {
        Some(avatar) => user_avatar_url(user.id, avatar, format, size),
        None => default_avatar_url(user.discriminator),
    }
}

/// Get a Discord user avatar link.
pub fn user_avatar_url(
    user_id: Id<UserMarker>,
    avatar: ImageHash,
    format: ImageFormat,
    size: ImageSize,
) -> String {
    format!("{USER_AVATAR_BASE}/{user_id}/{avatar}.{format}?size={size}")
}

/// Return the default avatar for a given discriminator.
///
/// The avatar is only available in PNG with a constant size.
pub fn default_avatar_url(discriminator: u16) -> String {
    // Number of the default avatar.
    let avatar_number = discriminator % 5;

    format!("{DEFAULT_AVATAR_BASE}/{avatar_number}.png")
}

/// Format of a Discord image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Webp,
    Gif,
}

impl Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Png => f.write_str("png"),
            ImageFormat::Jpeg => f.write_str("jpeg"),
            ImageFormat::Webp => f.write_str("webp"),
            ImageFormat::Gif => f.write_str("gif"),
        }
    }
}

/// Size of a Discord image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
    Size16,
    Size32,
    Size64,
    Size128,
    Size256,
    Size512,
    Size1024,
    Size2048,
    Size4096,
}

impl Display for ImageSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageSize::Size16 => f.write_str("16"),
            ImageSize::Size32 => f.write_str("32"),
            ImageSize::Size64 => f.write_str("64"),
            ImageSize::Size128 => f.write_str("128"),
            ImageSize::Size256 => f.write_str("256"),
            ImageSize::Size512 => f.write_str("512"),
            ImageSize::Size1024 => f.write_str("1024"),
            ImageSize::Size2048 => f.write_str("2048"),
            ImageSize::Size4096 => f.write_str("4096"),
        }
    }
}

/// Discord timestamp display style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampStyle {
    ShortTime,
    LongTime,
    ShortDate,
    LongDate,
    ShortDateTime,
    LongDateTime,
    RelativeTime,
}

impl TimestampStyle {
    /// Format a timestamp according to the current style.
    pub fn format(&self, timestamp: u64) -> String {
        format!("<t:{timestamp}:{self}>")
    }
}

impl Display for TimestampStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimestampStyle::ShortTime => f.write_str("t"),
            TimestampStyle::LongTime => f.write_str("T"),
            TimestampStyle::ShortDate => f.write_str("d"),
            TimestampStyle::LongDate => f.write_str("D"),
            TimestampStyle::ShortDateTime => f.write_str("f"),
            TimestampStyle::LongDateTime => f.write_str("F"),
            TimestampStyle::RelativeTime => f.write_str("R"),
        }
    }
}
