//! Models used in MongoDB collections.
//!
//! This module contains all models used in MongoDB database. These models can
//! be serialized and deserialized using `serde`.
//!
//! ## MongoDB collections
//! Here is the list of the different collections and their associated model:
//!
//! - `guilds` ([Guild]): configuration for guilds that uses the bot
//!
//! Each collection name is exported as a constant suffixed by `_COLLECTION`.
//!

mod guild;
pub mod serde_helpers;

pub use guild::{Config, Guild, GUILDS_COLLECTION};
