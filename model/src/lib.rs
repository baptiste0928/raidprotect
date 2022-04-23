//! # RaidProtect model
//!
//! This crate contains shared models between other workspace crates and
//! database connection wrappers.

pub mod cache;
pub mod collection;
pub mod interaction;
pub mod serde;

mod mongodb;
mod state;

pub use crate::mongodb::{MongoDbClient, MongoDbError};
pub use state::ClusterState;
