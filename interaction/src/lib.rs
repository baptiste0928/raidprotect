//! # RaidProtect interactions
//!
//! This crate contain types used to handle and respond to incoming Discord
//! interactions.

mod handle;

pub mod command;
pub mod component;
pub mod context;
pub mod embed;
pub mod response;

pub use handle::{handle_command, handle_component, register_commands};
