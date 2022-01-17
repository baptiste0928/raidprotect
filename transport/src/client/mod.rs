//! Gateway client.
//!
//! This module contain the client corresponding to the server of the [`server`]
//! module of this crate. See the server documentation for details about the
//! protocol.
//!
//! The inner client state is wrapped in an `RwLock<Arc>` and can be cloned
//! cheaply. Reconnection logic is handled internally.
//!
//! [`server`]: crate::server

#[allow(clippy::module_inception)]
mod client;
mod connect;

pub use client::GatewayClient;
