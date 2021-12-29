//! Requests and responses models.
//!
//! This module contains models for requests and responses
//! used by the client and the server.

use remoc::rch;
use serde::{Deserialize, Serialize};

/// Request sent over the base connection channel.
#[derive(Debug, Serialize, Deserialize)]
pub enum BaseRequest {
    /// Request an event broadcast channel.
    ///
    /// The channel will be sent using the provided callback channel.
    EventBroadcast {
        callback: rch::oneshot::Sender<EventBroadcastResponse>,
    },
}

/// Response of a [`BaseRequest::EventBroadcast`].
#[derive(Debug, Serialize, Deserialize)]
pub struct EventBroadcastResponse {}
