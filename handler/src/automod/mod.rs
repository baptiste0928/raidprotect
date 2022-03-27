//! Auto-moderation handler.
//!
//! This module handle auto-moderation features like anti-spam.

mod handle;

pub use handle::handle_message;
pub use raidprotect_analyzer::ALLOWED_MESSAGES_TYPES;
