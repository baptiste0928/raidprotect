//! # RaidProtect Gateway
//!
//! The gateway handle processing of incoming events received from Discord.
//! Events are used to update the cache and are forwarded to the appropriate
//! handle, with related data like cached guild and configuration injected..

mod cluster;

pub mod event;

pub use cluster::{ClusterError, ClusterState, ShardCluster};
