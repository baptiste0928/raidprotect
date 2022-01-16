//! Discord shards cluster.
//!
//! This module expose the [`ShardCluster`] type that handle websocket connection
//! to the Discord gateway and processing of incoming events.
//!
//! Incoming events are used to update the cache (see the [`cache`] module) and
//! are broadcasted to other services if needed. Only useful events are sent
//! to improve performances.
//!
//! [`cache`]: crate::cache

mod event;
mod shard;

pub use shard::ShardCluster;
