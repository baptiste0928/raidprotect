//! # RaidProtect model
//!
//! This crate contains models that represent the state of the bot.
//!
//! - Persistent state (guild configuration, moderation logs, ...) is stored in
//! a MongoDB database. See [`mongodb`].
//! - Cache and temporary state (pending components, ...) is stored in a Redis
//! database. See [`cache`].

mod serde;

pub mod cache;
pub mod mongodb;
