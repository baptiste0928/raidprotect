//! # RaidProtect handler
//!
//! This crate contain code used to handle events to perform user-side actions.
//! Unlike the `gateway` crate, it does not contain any code used to update the
//! bot internal state.
//!
//! ## Modules
//! The crate is split into different modules that correspond to the multiple
//! features available to the users.
//!
//! - [`embed`]: response embeds models
//! - [`interaction`]: interaction handling, using `twilight-interactions`

pub mod embed;
pub mod interaction;

pub mod translations {
    //! Translations loaded using [`rosetta_i18n`].

    rosetta_i18n::include_translations!();
}
