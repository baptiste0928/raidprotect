//! Gateway client.
//!
//! This module contain the client for the server exposed by the gateway. See*
//! the server documentation for details about the protocol.
//!
//! The inner client state is wrapped in an `RwLock<Arc>` and can be cloned
//! cheaply. Reconnection logic is handled internally.
#![allow(unused)]

mod cache;
#[allow(clippy::module_inception)]
mod client;
mod connect;
mod error;

pub use client::GatewayClient;
pub use error::ClientError;
