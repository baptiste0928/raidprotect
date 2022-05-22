//! # RaidProtect state
//!
//! This crate expose the [`ClusterState`] that hold the shared bot state. This
//! type is in a dedicated crate to avoid circular crate dependencies.

use std::sync::Arc;

use raidprotect_model::{mongodb::MongoDbClient, cache::{RedisClient, http::CacheHttp}};
use twilight_http::Client as HttpClient;
use twilight_model::id::{
    marker::{ApplicationMarker, GuildMarker},
    Id,
};

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
    /// Bot user id
    current_user: Id<ApplicationMarker>,
}

impl ClusterState {
    /// Initialize a new [`ClusterState`].
    pub fn new(
        redis: RedisClient,
        mongodb: MongoDbClient,
        http: Arc<HttpClient>,
        current_user: Id<ApplicationMarker>,
    ) -> Self {
        Self {
            redis,
            mongodb,
            http,
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

    /// Get the cluster [`CacheHttp`]
    pub fn cache_http(&self, guild_id: Id<GuildMarker>) -> CacheHttp {
        self.redis.http(&self.http, guild_id)
    }

    /// Get the bot user id
    pub fn current_user(&self) -> Id<ApplicationMarker> {
        self.current_user
    }
}
