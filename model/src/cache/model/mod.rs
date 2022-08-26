//! Cache models.
//!
//! This module contains models used by the cache. These models are based on
//! [`twilight_model`] models but without unnecessary fields to decrease memory
//! usage.
//!
//! Every model implement the [`Serialize`] and [`Deserialize`] traits.
//!
//! [`Serialize`]: serde::Serialize
//! [`Deserialize`]: serde::Deserialize

pub mod interaction;
pub mod message;

mod channel;
mod guild;

pub use channel::CachedChannel;
pub use guild::{CachedGuild, CachedRole, CurrentMember};
