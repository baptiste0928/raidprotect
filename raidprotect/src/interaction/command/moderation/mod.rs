//! Moderation commands
//!
//! This module contains the `kick`, `warn`, `ban` and `mute` commands of
//! RaidProtect. These moderation commands have a similar behavior and share
//! functions to avoid duplication.
//!
//! ## Handling moderation commands
//! When a moderation command is received, the bot first check if the user that
//! have done the command has the required permissions to perform the action
//! on the targeted user regarding of the role hierarchy.
//!
//! Then, if no reason is provided with the optional `reason` parameter of each
//! command, a modal is shown to let the user enter a reason and internal notes
//! for the sanction.
//!
//! The sanctioned user receive a private message with the reason, and the
//! sanction is applied. It is also logged in the guild's logs channel and in
//! the bot database.

mod kick;

pub use kick::KickCommand;
