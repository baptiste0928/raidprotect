//! Cache models.
//!
//! This module contains models used by the cache. These models
//! are based on [`twilight_model`] models but without unnecessary
//! fields to decrease memory usage.

mod channel;
mod guild;

pub use channel::{CachedCategoryChannel, CachedChannel, CachedTextChannel, CachedThread};
pub use guild::{CachedGuild, CachedRole, CurrentMember};
