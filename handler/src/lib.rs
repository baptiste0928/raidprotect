//! # RaidProtect handler
//!
//! This crate contain code used to handle events to perform user-side actions.
//! Unlike the `gateway` crate, it does not contain any code used to update the
//! bot internal state.
//!
//! ## Modules
//! The crate is splitted into different modules that correspond to the multiple
//! features avaiable to the users.
//!
//! - [`embed`]: response embeds models
//! - [`interaction`]: interaction handling, using `twilight-interactions`

pub mod embed;
pub mod interaction;
