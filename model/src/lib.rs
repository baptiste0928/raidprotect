//! # RaidProtect model
//!
//! This crate contains shared models between other workspace crates and
//! database connection wrappers.

pub mod collection;
pub mod interaction;
pub mod mongodb;
mod state;

pub use state::ClusterState;
