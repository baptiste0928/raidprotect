//! Slash commands handler.

mod handle;

pub mod callback;
pub mod help;

pub use handle::{handle, IntoCallback};
