//! Custom cache used to store Discord objects.
//!
//! This cache is based on Redis and store Discord objects used by the bot
//! including guilds, channels and roles. The cache is built to use as little
//! memory as possible, and such only store useful fields.
//!
//! ## Access the cache data
//! The cache can be queried using [`RedisClient`]. Higher-level interfaces are
//! also provided to use the cache data: the [`permission`] allow to compute
//! permissions for a user using cached data, and [`http`] allow to perform
//! permission checks before http requests.
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

mod process;

pub mod http;
pub mod model;
pub mod permission;
mod redis;

pub use self::{
    process::UpdateCache,
    redis::{RedisClient, RedisConnection, RedisModel},
};
