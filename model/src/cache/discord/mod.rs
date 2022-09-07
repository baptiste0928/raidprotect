//! Discord cache models and event processing.
//!
//! This module contains all types representing Discord cached objects and the
//! functions to process incoming events into the cache. These models are based
//! on [`twilight_model`] models but without unnecessary fields to decrease memory
//! usage.
//!
//! ## Event processing
//! Incoming Discord events that implement [`UpdateCache`] are processed to
//! update the cache. The old cached value is returned after updating.
//!
//! The following events are used to update the cache:
//!
//! | Cached data           | Event types                                                       |
//! |-----------------------|-------------------------------------------------------------------|
//! | Guilds                | `GuildCreate`, `GuildUpdate`, `GuildDelete`, `UnavailableGuild`   |
//! | Channels (guild-only) | `ChannelCreate`, `ChannelUpdate`, `ChannelUpdate` (+ thread ones) |
//! | Roles                 | `RoleCreate`, `RoleUpdate`, `RoleDelete`                          |
//! | Current user member   | `MemberAdd`, `MemberUpdate`                                       |
//!
//! [`Serialize`]: serde::Serialize
//! [`Deserialize`]: serde::Deserialize

mod client;
mod model;
mod process;

pub mod http;
pub mod permission;

pub use model::{
    channel::CachedChannel,
    guild::{CachedGuild, CachedRole, CurrentMember},
};
pub use process::event::UpdateCache;
