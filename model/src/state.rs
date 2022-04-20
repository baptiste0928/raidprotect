//! Models used to store the bot state.

use std::sync::Arc;

use raidprotect_cache::{InMemoryCache, MessageCache};
use twilight_http::Client as HttpClient;

use crate::{interaction::component::PendingComponentQueue, mongodb::MongoDbClient};

/// Current state of the cluster.
///
/// This type hold shared types such as the cache or the http client. It does
/// not implement [`Clone`] and is intended to be wrapped inside a [`Arc`].
#[derive(Debug)]
pub struct ClusterState {
    /// In-memory cache
    cache: InMemoryCache,
    /// MongoDB client
    mongodb: MongoDbClient,
    /// Http client
    http: Arc<HttpClient>,
    /// Message cache client
    messages: MessageCache,
    /// Pending components queue
    pending_components: PendingComponentQueue,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(
        cache: InMemoryCache,
        mongodb: MongoDbClient,
        http: Arc<HttpClient>,
        messages: MessageCache,
        pending_components: PendingComponentQueue,
    ) -> Self {
        Self {
            cache,
            mongodb,
            http,
            messages,
            pending_components,
        }
    }

    /// Get the cluster [`InMemoryCache`].
    pub fn cache(&self) -> &InMemoryCache {
        &self.cache
    }

    /// Get the cluster [`MongoDbClient`].
    pub fn mongodb(&self) -> &MongoDbClient {
        &self.mongodb
    }

    /// Get the cluster [`HttpClient`].
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the cluster [`MessageCache`].
    pub fn messages(&self) -> &MessageCache {
        &self.messages
    }

    /// Get the cluster [`PendingComponentQueue`].
    pub fn pending_components(&self) -> &PendingComponentQueue {
        &self.pending_components
    }
}
