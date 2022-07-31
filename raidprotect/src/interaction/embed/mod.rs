//! Response embed models.
//!
//! This crate contains types used to generate embeds used as bot responses.

pub mod captcha;
pub mod error;
pub mod kick;

/// RaidProtect's red color.
pub const COLOR_RED: u32 = 0xd35f5f;

/// Transparent embed color (dark theme)
pub const COLOR_TRANSPARENT: u32 = 0x2f3136;

/// Light green color.
///
/// Used for success messages, like after a configuration change.
pub const COLOR_GREEN: u32 = 0xa0d995;
