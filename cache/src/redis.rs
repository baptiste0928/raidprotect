use std::time::Duration;

use bb8::{Pool, RunError, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::RedisError;
use thiserror::Error;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::model::CachedGuild;

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
    pub async fn new(uri: &str) -> Result<Self, RedisClientError> {
        let manager = RedisConnectionManager::new(uri)?;
        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(2))
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    /// Get a new connection from the connection pool
    pub async fn conn(&self) -> Result<PooledConnection<'_, RedisConnectionManager>, RedisClientError> {
        Ok(self.pool.get().await?)
    }

    pub async fn guild(&self, _id: Id<GuildMarker>) -> Result<Option<CachedGuild>, RedisClientError> {
        todo!()
    }
}

/// Error type returned by [`RedisClient`] methods.
#[derive(Debug, Error)]
pub enum RedisClientError {
    #[error(transparent)]
    Redis(#[from] RedisError),
    #[error("connection pool timed out")]
    TimedOut,
}

impl From<RunError<RedisError>> for RedisClientError {
    fn from(error: RunError<RedisError>) -> Self {
        match error {
            RunError::User(error) => Self::from(error),
            RunError::TimedOut => Self::TimedOut,
        }
    }
}
