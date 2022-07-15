//! # RaidProtect interactions
//!
//! This crate contain types used to handle and respond to incoming Discord
//! interactions.

mod handle;

pub mod command;
pub mod component;
pub mod embed;
pub mod response;
pub mod util;

pub use handle::{handle_interaction, register_commands};
