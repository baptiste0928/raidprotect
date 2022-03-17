//! Models used to store the bot state.

use std::sync::Arc;

use raidprotect_cache::InMemoryCache;
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
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(cache: InMemoryCache, http: Arc<HttpClient>) -> Self {
        Self { cache, http }
    }

    /// Get the cluster [`InMemoryCache`].
    pub fn cache(&self) -> &InMemoryCache {
        &self.cache
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> &HttpClient {
        &self.http
    }
}
