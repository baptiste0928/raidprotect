//! Database client and models.
//!
//! This module contains models used to represent data in the MongoDB database
//! and the [`DbClient`] connection wrapper.
//!
//! ## MongoDB collections
//! The following collections are used:
//! - `guilds` ([GuildConfig]): configuration for guilds that uses the bot
//! - `modlogs` ([Modlog]): moderation logs
//!
//! Each collection name is exported as an associated constant.
//!
//! [GuildConfig]: guild::GuildConfig
//! [Modlog]: modlog::Modlog

mod client;
mod guild;
mod modlog;

pub use client::DbClient;

pub mod model {
    //! Models used to represent data in the MongoDB database.
    //!
    //! See the [module documentation](crate::database) for more information.

    pub use super::{
        guild::{CaptchaConfig, GuildConfig, ModerationConfig},
        modlog::{Modlog, ModlogType, ModlogUser},
    };
}
