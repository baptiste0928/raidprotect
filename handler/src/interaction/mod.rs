//! Slash commands handler.

mod handle;

pub mod callback;
// pub mod help;
pub mod context;

pub use handle::handle_command;
