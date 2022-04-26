//! Redis client.
//!
//! This module expose the [`RedisClient] type used to access the cache stored
//! in Redis.

use std::time::Duration;

use bb8::{Pool, PooledConnection, RunError};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, ErrorKind, FromRedisValue, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::model::{CachedChannel, CachedGuild, CachedRole};

/// Alias for a [`Result`] with [`RedisClientError`] as error type.
pub type RedisResult<T> = Result<T, RedisClientError>;

/// Alias for Redis connection type.
pub type RedisConnection<'a> = PooledConnection<'a, RedisConnectionManager>;

/// Redis client.
///
/// This type wrap a Redis connection pool and can be cloned.
#[derive(Debug, Clone)]
pub struct RedisClient {
    /// Internal connection pool.
    pool: Pool<RedisConnectionManager>,
}

impl RedisClient {
    /// Initialize a new [`RedisClient`].
    pub async fn new(uri: &str) -> RedisResult<Self> {
        let manager = RedisConnectionManager::new(uri)?;
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(2))
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    /// Get a new connection from the connection pool
    pub async fn conn(&self) -> RedisResult<RedisConnection<'_>> {
        Ok(self.pool.get().await?)
    }

    pub async fn get<T: RedisModel>(&self, id: &T::Id) -> RedisResult<Option<T>> {
        let mut conn = self.conn().await?;

        Ok(conn.get(T::key_from(id)).await?)
    }

    /// Get all the [`CachedChannel`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_channels(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedChannel>> {
        let mut conn = self.conn().await?;
        let guild: Option<CachedGuild> = conn.get(CachedGuild::key_from(&id)).await?;

        if let Some(guild) = guild {
            let mut pipe = redis::pipe();

            for channel in guild.channels {
                pipe.get(CachedChannel::key_from(&channel));
            }

            Ok(pipe.query_async(&mut *conn).await?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all the [`CachedRole`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_roles(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedRole>> {
        let mut conn = self.conn().await?;
        let guild: Option<CachedGuild> = conn.get(CachedGuild::key_from(&id)).await?;

        if let Some(guild) = guild {
            let mut pipe = redis::pipe();

            for role in guild.roles {
                pipe.get(CachedRole::key_from(&role));
            }

            Ok(pipe.query_async(&mut *conn).await?)
        } else {
            Ok(Vec::new())
        }
    }
}

/// This trait is implemented by types representing a Redis model.
///
/// It provides methods to get the model key used
/// in Redis, as well as a default implement of [`ToRedisArgs`] and
/// [`FromRedisValue`].
pub trait RedisModel: Serialize + DeserializeOwned {
    /// Type used for the unique model identifier.
    type Id;

    /// Get the current value key.
    fn key(&self) -> String;

    /// Get the key for this model from a unique id.
    fn key_from(id: &Self::Id) -> String;

    /// Serialize this model.
    ///
    /// The default implementation serialize the model in MessagePack using
    /// [`rmp_serde`] and compress it with [`zstd`].
    fn serialize(&self) -> RedisResult<Vec<u8>> {
        let serialized = rmp_serde::to_vec(self)?;

        zstd::encode_all(&*serialized, 0).map_err(RedisClientError::Zstd)
    }

    /// Deserialize this model.
    ///
    /// The default implementation decompress the model with [`zstd`] and
    /// deserialize it from MessagePack with [`rmp_serde`].
    fn deserialize(value: &[u8]) -> RedisResult<Self> {
        let decoded = zstd::decode_all(value).map_err(RedisClientError::Zstd)?;

        Ok(rmp_serde::from_slice(&decoded)?)
    }
}

impl<T: RedisModel> FromRedisValue for T {
    fn from_redis_value(value: &redis::Value) -> Result<Self, RedisError> {
        let data = match value {
            redis::Value::Data(data) => data,
            _ => {
                return Err(RedisError::from((
                    ErrorKind::TypeError,
                    "response was not binary data",
                )))
            }
        };

        Self::deserialize(&*data).map_err(|err| {
            RedisError::from((ErrorKind::TypeError, "invalid response", err.to_string()))
        })
    }
}

/// Error type returned by [`RedisClient`] methods.
#[derive(Debug, Error)]
pub enum RedisClientError {
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error("connection pool timed out")]
    TimedOut,
    #[error("failed to serialize data: {0}")]
    Serialize(#[from] rmp_serde::encode::Error),
    #[error("failed to deserialize data: {0}")]
    Deserialize(#[from] rmp_serde::decode::Error),
    #[error("error with zstd compression: {0}")]
    Zstd(#[source] std::io::Error),
}

impl From<RunError<RedisError>> for RedisClientError {
    fn from(error: RunError<RedisError>) -> Self {
        match error {
            RunError::User(error) => Self::from(error),
            RunError::TimedOut => Self::TimedOut,
        }
    }
}
