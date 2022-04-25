//! Redis client.
//!
//! This module expose the [`RedisClient] type used to access the cache stored
//! in Redis.

use std::time::Duration;

use bb8::{Pool, PooledConnection, RunError};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker},
    Id,
};

use crate::model::{CachedChannel, CachedGuild, CachedRole};

/// Alias for a [`Result`] with [`RedisClientError`] as error type.
pub type RedisResult<T> = Result<T, RedisClientError>;

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
    pub async fn conn(&self) -> RedisResult<PooledConnection<'_, RedisConnectionManager>> {
        Ok(self.pool.get().await?)
    }

    /// Serialize a value before storing in Redis.
    ///
    /// The value is serialized in MessagePack using [`rmp_serde`] and compressed
    /// with [`zstd`].
    pub(crate) fn serialize<T: Serialize>(value: &T) -> RedisResult<Vec<u8>> {
        let serialized = rmp_serde::to_vec(value)?;

        zstd::encode_all(serialized.as_slice(), 0).map_err(RedisClientError::Zstd)
    }

    /// Deserialize a value before storing in Redis.
    ///
    /// The value is decompressed with [`zstd`] and deserialized from MessagePack
    /// using [`rmp_serde`].
    pub(crate) fn deserialize<T: DeserializeOwned>(value: impl AsRef<[u8]>) -> RedisResult<T> {
        let serialized = zstd::decode_all(value.as_ref()).map_err(RedisClientError::Zstd)?;

        Ok(rmp_serde::from_slice(&serialized)?)
    }

    /// Get a [`CachedGuild`] by id.
    pub async fn guild(&self, id: Id<GuildMarker>) -> RedisResult<Option<CachedGuild>> {
        let mut conn = self.conn().await?;
        let req: Option<Vec<u8>> = conn.get(format!("c:guild:{}", id)).await?;

        req.map(Self::deserialize).transpose()
    }

    /// Get a [`CachedChannel`] by id.
    pub async fn channel(&self, id: Id<ChannelMarker>) -> RedisResult<Option<CachedChannel>> {
        let mut conn = self.conn().await?;
        let req: Option<Vec<u8>> = conn.get(format!("c:channel:{}", id)).await?;

        req.map(Self::deserialize).transpose()
    }

    /// Get a [`CachedRole`] by id.
    pub async fn role(&self, id: Id<RoleMarker>) -> RedisResult<Option<CachedRole>> {
        let mut conn = self.conn().await?;
        let req: Option<Vec<u8>> = conn.get(format!("c:role:{}", id)).await?;

        req.map(Self::deserialize).transpose()
    }

    /// Get all the [`CachedChannel`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_channels(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedChannel>> {
        let mut conn = self.conn().await?;
        let req: Option<Vec<u8>> = conn.get(format!("c:guild:{}", id)).await?;
        let guild: Option<CachedGuild> = req.map(Self::deserialize).transpose()?;

        if let Some(guild) = guild {
            let mut pipe = redis::pipe();

            for channel in guild.channels {
                pipe.get(format!("c:channel:{}", channel));
            }

            let channels: Vec<Vec<u8>> = pipe.query_async(&mut *conn).await?;
            if !channels.is_empty() {
                return channels.iter().map(Self::deserialize).collect();
            }
        }

        Ok(Vec::new())
    }

    /// Get all the [`CachedRole`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_roles(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedRole>> {
        let mut conn = self.conn().await?;
        let req: Option<Vec<u8>> = conn.get(format!("c:guild:{}", id)).await?;
        let guild: Option<CachedGuild> = req.map(Self::deserialize).transpose()?;

        if let Some(guild) = guild {
            let mut pipe = redis::pipe();

            for role in guild.roles {
                pipe.get(format!("c:role:{}", role));
            }

            let roles: Vec<Vec<u8>> = pipe.query_async(&mut *conn).await?;
            if !roles.is_empty() {
                return roles.iter().map(Self::deserialize).collect();
            }
        }

        Ok(Vec::new())
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
