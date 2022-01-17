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
#![allow(unused)]

#[allow(clippy::module_inception)]
mod client;
mod connect;
mod error;

pub use client::GatewayClient;
pub use error::ClientError;
