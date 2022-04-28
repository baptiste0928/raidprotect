//! # RaidProtect state
//!
//! This crate expose the [`ClusterState`] that hold the shared bot state. This
//! type is in a dedicated crate to avoid circular crate dependencies.

use std::sync::Arc;

use raidprotect_cache::{redis::RedisClient, MessageCache};
use raidprotect_model::{interaction::component::PendingComponentQueue, mongodb::MongoDbClient};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::ApplicationMarker, Id};

/// Current state of the cluster.
///
/// This type hold shared types such as the cache or the http client. It does
/// not implement [`Clone`] and is intended to be wrapped inside a [`Arc`].
#[derive(Debug)]
pub struct ClusterState {
    /// In-memory cache
    redis: RedisClient,
    /// MongoDB client
    mongodb: MongoDbClient,
    /// Http client
    http: Arc<HttpClient>,
    /// Message cache client
    messages: MessageCache,
    /// Pending components queue
    pending_components: PendingComponentQueue,
    /// Bot user id
    current_user: Id<ApplicationMarker>,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(
        redis: RedisClient,
        mongodb: MongoDbClient,
        http: Arc<HttpClient>,
        messages: MessageCache,
        pending_components: PendingComponentQueue,
        current_user: Id<ApplicationMarker>,
    ) -> Self {
        Self {
            redis,
            mongodb,
            http,
            messages,
            pending_components,
            current_user,
        }
    }

    /// Get the cluster [`RedisClient`].
    pub fn redis(&self) -> &RedisClient {
        &self.redis
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

    /// Get the bot user id
    pub fn current_user(&self) -> Id<ApplicationMarker> {
        self.current_user
    }
}
