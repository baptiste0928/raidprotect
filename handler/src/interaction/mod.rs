//! Slash commands handler.

mod handle;

pub mod help;

pub use handle::{handle, IntoCallback};
