//! MongoDB client and collection models.
//!
//! This module contains all models used in MongoDB database and the [`MongoDbClient`]
//! connection wrapper. The models can be serialized and deserialized using `serde`.
//!
//! ## MongoDB collections
//! - `guilds` ([Guild]): configuration for guilds that uses the bot
//! - `modlogs` ([Modlog]): moderation logs
//!
//! Each collection name is exported as an associated constant.
//!
//! [Guild]: guild::Guild
//! [Modlog]: modlog::Modlog

mod client;
pub mod guild;
pub mod modlog;

pub use client::{MongoDbClient, MongoDbError};
