//! Redis client.
//!
//! This module expose the [`CacheClient`] type used to access the cache stored
//! in Redis.

use std::{fmt::Debug, time::Duration};

use anyhow::Context;
use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{instrument, trace};

/// Alias for Redis connection type.
pub type RedisConnection<'a> = PooledConnection<'a, RedisConnectionManager>;

/// Wrapper around a Redis connection pool.
///
/// This type wraps an underlying Redis connection pool and exposes high-level
/// methods to access data stored in the cache.
///
/// It can be cheaply cloned because the underlying [`Pool`] uses [`Arc`].
///
/// [`Arc`]: std::sync::Arc
#[derive(Debug, Clone)]
pub struct CacheClient {
    /// Internal connection pool.
    pool: Pool<RedisConnectionManager>,
}

impl CacheClient {
    /// Connects to Redis and returns the client.
    pub async fn connect(uri: &str) -> Result<Self, anyhow::Error> {
        let manager =
            RedisConnectionManager::new(uri).context("failed to initialize connection manager")?;

        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(2))
            .build(manager)
            .await
            .context("failed to initialize connection pool")?;

        Ok(Self { pool })
    }

    /// Returns a new [`RedisConnection`] from the pool.
    pub async fn conn(&self) -> Result<RedisConnection<'_>, anyhow::Error> {
        Ok(self.pool.get().await?)
    }

    /// Run a `PING` command to check if the cache is connected.
    pub async fn ping(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.conn().await?;
        redis::cmd("PING").query_async(&mut *conn).await?;

        Ok(())
    }

    /// Get a value from the cache.
    #[instrument(skip(self))]
    pub async fn get<T: RedisModel>(&self, id: &T::Id) -> Result<Option<T>, anyhow::Error> {
        let mut conn = self.conn().await?;
        let key = T::key_from(id);

        trace!("getting value for key {}", key);
        let value: Option<_> = conn.get(&key).await?;

        value.map(RedisModel::deserialize_model).transpose()
    }

    /// Set a value in the cache.
    #[instrument(skip(self))]
    pub async fn set<T: RedisModel>(&self, value: &T) -> Result<(), anyhow::Error> {
        let mut conn = self.conn().await?;
        let key = value.key();

        trace!(value = ?value, "setting value for key {}", key);
        if let Some(expires_after) = T::EXPIRES_AFTER {
            conn.set_ex(value.key(), value.serialize_model()?, expires_after)
                .await?;
        } else {
            conn.set(value.key(), value.serialize_model()?).await?;
        }

        Ok(())
    }

    /// Delete a value from the cache.
    #[instrument(skip(self))]
    pub async fn delete<T: RedisModel>(&self, value: &T) -> Result<(), anyhow::Error> {
        let mut conn = self.conn().await?;
        let key = value.key();

        trace!("deleting value for key {}", key);
        conn.del(key).await?;

        Ok(())
    }
}

/// Type representing a model stored in the cache.
///
/// It provides methods to get the model key used in Redis, as well as methods
/// for serialization and deserialization.
pub trait RedisModel: Debug + Serialize + DeserializeOwned {
    /// Type of the model ID.
    type Id: ?Sized + Debug;

    /// Default key expiration delay.
    ///
    /// If set to `None`, the key never expires.
    const EXPIRES_AFTER: Option<usize> = None;

    /// Get the current value key.
    fn key(&self) -> String;

    /// Get the key for this model from a unique id.
    fn key_from(id: &Self::Id) -> String;

    /// Serialize this model.
    ///
    /// The default implementation serializes the model in MessagePack using
    /// [`rmp_serde`].
    fn serialize_model(&self) -> Result<Vec<u8>, anyhow::Error> {
        let serialized = rmp_serde::to_vec_named(self)?;
        trace!(value = ?self, serialized = ?serialized, "serializing model");

        Ok(serialized)
    }

    /// Deserialize this model.
    ///
    /// The default implementation deserializes the model from MessagePack with
    /// [`rmp_serde`].
    fn deserialize_model(value: Vec<u8>) -> Result<Self, anyhow::Error> {
        trace!(value = ?value, "deserializing model");

        Ok(rmp_serde::from_slice(&value)?)
    }
}
