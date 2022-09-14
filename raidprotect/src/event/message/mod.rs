//! Message event handling.
//!
//! This module contain logic used to handle incoming message, such as spam
//! detection.

mod handle;
mod old_command;

pub mod parser;

pub use handle::{handle_message_create, handle_message_delete};

/// Messages types processed by the bot.
pub const ALLOWED_MESSAGES_TYPES: [twilight_model::channel::message::MessageType; 3] = [
    twilight_model::channel::message::MessageType::Regular,
    twilight_model::channel::message::MessageType::Reply,
    twilight_model::channel::message::MessageType::ThreadStarterMessage,
];
