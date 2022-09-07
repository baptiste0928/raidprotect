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

pub mod channel;
pub mod guild;
