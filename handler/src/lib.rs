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
//! - [`command`]: slash command handling, using `twilight-interactions`

mod command;
