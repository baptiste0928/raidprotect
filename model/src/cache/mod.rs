//! Cache models.
//!
//! This module contains models used by the cache. These models
//! are based on [`twilight_model`] models but without unnecessary
//! fields to decrease memory usage.

mod guild;

pub mod channel;
pub mod role;

pub use guild::{CachedGuild, CurrentMember};
