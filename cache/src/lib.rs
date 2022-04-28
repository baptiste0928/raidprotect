//! # raidprotect-cache
//!
//! Implementation of the custom cache used to store Discord objects.
//!
//! This cache is based on Redis and store Discord objects used by the bot
//! including guilds, channels and roles. The cache is built to use as little
//! memory as possible, and such only store useful fields.
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

mod message;
mod process;

pub mod model;
pub mod permission;
pub mod redis;

pub use message::{MessageCache, MessageExpireTask};
pub use process::UpdateCache;
