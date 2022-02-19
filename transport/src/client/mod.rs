//! Gateway client.
//!
//! This module contain the client for the server exposed by the gateway. See*
//! the server documentation for details about the protocol.
//!
//! The inner client state is wrapped in an `RwLock<Arc>` and can be cloned
//! cheaply. Reconnection logic is handled internally.

mod cache;
mod connect;
mod error;
mod gateway;

pub use error::ClientError;
pub use gateway::{GatewayAddr, GatewayClient};
