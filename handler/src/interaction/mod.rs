//! Slash commands handler.

mod handle;

pub mod command;
pub mod context;
pub mod response;

pub use handle::handle_command;
