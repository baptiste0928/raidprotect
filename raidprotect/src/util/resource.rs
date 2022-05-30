//! Utility functions to format Discord resources.

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
pub fn avatar_url(user: &User, format: &str, size: u16) -> String {
    debug_assert!(
        ["webp", "png", "gif", "jpg"].contains(&format),
        "invalid avatar format"
    );
    debug_assert!(
        (16..=4096).contains(&size),
        "size must be between 16 and 4096"
    );
    debug_assert!(size & (size - 1) == 0, "size must be a power of 2");

    match user.avatar {
        Some(avatar) => user_avatar_url(user.id, avatar, format, size),
        None => default_avatar_url(user.discriminator),
    }
}

/// Get a Discord user avatar link.
pub fn user_avatar_url(
    user_id: Id<UserMarker>,
    avatar: ImageHash,
    format: &str,
    size: u16,
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
