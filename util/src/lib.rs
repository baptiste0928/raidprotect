//! Utility modules used across RaidProtect crates.
//!
//! This crate is used to expose utility modules that are shared between multiple
//! crates, such as the `event` and `interaction`crates
//!
//! It actually provide the following features :
//! - [`logs_channel`]: get the logs channel of a guild
//! - [`resource`]: format discord resources such as avatar links
//! - [`text`]: extension traits for text transformation

pub mod logs_channel;
pub mod resource;
pub mod text;

/// RaidProtect's red color.
pub const COLOR_RED: u32 = 0xd35f5f;

/// Transparent embed color (dark theme)
pub const COLOR_TRANSPARENT: u32 = 0x2f3136;
