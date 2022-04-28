//! Utility modules used across RaidProtect crates.
//!
//! This crate is used to expose utility modules that does not fit in other
//! crates or need to be shared between multiple crates.
//!
//! It actually provide the following features :
//! - [`resource`]: format discord resources such as avatar links
//! - [`logging`]: utility functions to setup consistent logging across crates
//! - [`serde`]: custom serde helpers
//! - [`shutdown`]: types used to manage tasks graceful shutdown
//! - [`text`]: extension traits for text transformation

pub mod logging;
pub mod resource;
pub mod serde;
pub mod shutdown;
pub mod text;
