//! Shared bot cache.
//!
//! This cache is based on Redis and store temporary data for the bot, such as the
//! Discord cache. It is built to use as little memory as possible to handle millions
//! of cached items.
//!
//! ## Access the cache data
//! The cache can be queried using [`CacheClient`] and the model of the requested
//! data. Higher-level interfaces are provided to use the cache data in the
//! [`discord`] module, such as a permission calculator and a Discord http client
//! wrapper.
//!
//! Models of cached data can be found in the [`model`] module and the [`discord`]
//! module (for cached Discord data). All models implements the [`RedisModel`]
//! trait to be serializable in the cache.

pub mod discord;
pub mod model;

mod client;

pub use self::client::{CacheClient, RedisConnection, RedisModel};
