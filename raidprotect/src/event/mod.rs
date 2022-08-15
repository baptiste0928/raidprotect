//! Gateway event processing.
//!
//! This module handle processing of incoming gateway events. This processing
//! only include updating the cache and retrieving information from the database.
//!
//! The user-side event handling is done in the `raidprotect_handler` crate.

mod captcha;
mod message;
mod process;

pub use process::ProcessEvent;
