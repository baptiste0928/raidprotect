//! MongoDB client and collection models.
//!
//! This module contains all models used in MongoDB database and the [`MongoDbClient`]
//! connection wrapper. The models can be serialized and deserialized using `serde`.
//!
//! ## MongoDB collections
//! - `guilds` ([Guild]): configuration for guilds that uses the bot
//!
//! Each collection name is exported as an associated constant.

mod client;
mod guild;
mod modlog;

pub use client::{MongoDbClient, MongoDbError};
pub use guild::{Config, Guild};
pub use modlog::{Modlog, ModlogKind, ModlogUser};
