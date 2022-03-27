//! Models used to store the bot state.

use std::sync::Arc;

use raidprotect_cache::{InMemoryCache, MessageCache};
use twilight_http::Client as HttpClient;

/// Current state of the cluster.
///
/// This type hold shared types such as the cache or the http client. It does
/// not implement [`Clone`] and is intended to be wrapped inside a [`Arc`].
#[derive(Debug)]
pub struct ClusterState {
    /// In-memory cache
    cache: InMemoryCache,
    /// Http client
    http: Arc<HttpClient>,
    /// Message cache client
    messages: Arc<MessageCache>,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(cache: InMemoryCache, http: Arc<HttpClient>, messages: Arc<MessageCache>) -> Self {
        Self {
            cache,
            http,
            messages,
        }
    }

    /// Get the cluster [`InMemoryCache`].
    pub fn cache(&self) -> &InMemoryCache {
        &self.cache
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the cluster [`MessageCache`].
    pub fn messages(&self) -> &MessageCache {
        self.messages.as_ref()
    }
}
