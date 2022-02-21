//! Gateway event processing.
//!
//! This module handle processing of incoming gateway events. This processing
//! only include updating the cache and retreiving information from the database.
//!
//! The user-side event handling is done in the [`raidprotect_handler`] crate.

pub mod model;
mod process;

pub use process::ProcessEvent;
