//! Utility modules used by the bot.
//!
//! This module provides various utilities that doesn't fit in other modules.

mod logs_channel;
pub mod resource;
pub mod shutdown;
mod text;

pub use logs_channel::guild_logs_channel;
pub use text::TextProcessExt;
