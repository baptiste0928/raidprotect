//! Utility modules used across RaidProtect crates.
//!
//! This crate is used to expose utility modules that are shared between multiple
//! crates, such as the `event` and `interaction`crates
//!
//! It actually provide the following features :
//! - [`resource`]: format discord resources such as avatar links
//! - [`text`]: extension traits for text transformation

pub mod resource;
pub mod text;
