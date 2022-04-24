//! Message event handling.
//!
//! This module contain logic used to handle incoming message, such as spam
//! detection.

mod handle;

pub mod parser;

pub use handle::handle_message;

use twilight_model::channel::message::MessageType;

/// Messages types processed by the bot.
pub const ALLOWED_MESSAGES_TYPES: [MessageType; 3] = [
    MessageType::Regular,
    MessageType::Reply,
    MessageType::ThreadStarterMessage,
];
