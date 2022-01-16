//! Requests and responses models.
//!
//! This module contains models for requests and responses
//! used by the client and the server.

use raidprotect_model::event::Event;
use remoc::rch;
use serde::{Deserialize, Serialize};

use crate::cache::CacheClient;

/// Request sent over the base connection channel.
#[derive(Debug, Serialize, Deserialize)]
pub enum BaseRequest {
    /// Request an event broadcast channel.
    ///
    /// The channel will be sent using the provided callback channel.
    EventBroadcast {
        callback: rch::oneshot::Sender<EventBroadcastResponse>,
    },
    /// Request a cache client.
    ///
    /// The cache client will be sent using the provided callback channel.
    Cache {
        callback: rch::oneshot::Sender<CacheResponse>,
    },
}

/// Response of a [`BaseRequest::EventBroadcast`].
#[derive(Debug, Serialize, Deserialize)]
pub struct EventBroadcastResponse {
    /// The requested event stream.
    pub events: rch::mpsc::Receiver<Event>,
}

/// Response of a [`BaseRequest::Cache`].
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheResponse {
    /// The requested cache client.
    pub client: CacheClient,
}
