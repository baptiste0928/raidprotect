//! Utility modules used across RaidProtect crates.
//!
//! This crate is used to expose utility modules that does not fit in other crates.
//! It actually provide the following features :
//! - [`shutdown`]: types used to manage tasks graceful shutdown
//! - [`logging`]: utility functions to setup consistent logging across crates

pub mod logging;
pub mod shutdown;
pub mod text;
