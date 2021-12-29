//! Gateway server.
//!
//! This gateway server exposes API used by other services to
//! communicate with the gateway :
//! - Remote trait client to query cached data
//! - Services can register to receive Discord events
//!
//! ## Protocol
//! The server is based on [`remoc`]. When a new TCP connection
//! is accepted, an initial [`remoc::rch::base`] channel is created.
//!
//! This base channel is used to initialize other specific channels
//! requested by the client.
//!
//! ## Shutdown handling
//! When the gateway shutdown, the following policy is applied :
//!
//! - **Immediately**, the main TCP listener stop accepting new connections.
//!   Existing connection base channels are closed.
//! - Cache client stop accepting request after 1.5 seconds and wait until all
//!   pending requests are finished.
//! - Event forwarding is stopped, and all pending request are finished.
//! - Each connection is closed when all sub-tasks are closed, and the main TCP
//!   listener waits for each connection to terminate before closing connection.
//!
//! Shutdown notifications are handled by the [`Shutdown`] type. MPSC channels are
//! used to track when all producers are dropped and know when tasks are finished.
//!
//! [`Shutdown`]: raidprotect_util::shutdown::Shutdown

mod handler;
mod listener;

pub use listener::GatewayListener;
