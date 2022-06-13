//! Redis client.
//!
//! This module expose the [`RedisClient] type used to access the cache stored
//! in Redis.

use core::fmt;
use std::time::Duration;

use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use error_stack::{Context, IntoReport, Report, ResultExt};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::GuildMarker, Id};

use super::{
    http::CacheHttp,
    model::{CachedChannel, CachedGuild, CachedRole},
    permission::{GuildPermissions, PermissionError},
};

/// Alias for a [`Result`] with [`RedisClientError`] as error type.
pub type RedisResult<T> = Result<T, Report<RedisClientError>>;

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
        let manager = RedisConnectionManager::new(uri)
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to initialize connection manager")?;

        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(2))
            .build(manager)
            .await
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to initialize connection pool")?;

        Ok(Self { pool })
    }

    /// Get a new connection from the connection pool
    pub async fn conn(&self) -> RedisResult<RedisConnection<'_>> {
        let conn = self
            .pool
            .get()
            .await
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to get connection from pool")?;

        Ok(conn)
    }

    /// Get a value from Redis.
    pub async fn get<T: RedisModel>(&self, id: &T::Id) -> RedisResult<Option<T>> {
        let mut conn = self.conn().await?;
        let value: Option<_> = conn
            .get(T::key_from(id))
            .await
            .report()
            .change_context(RedisClientError)
            .attach_printable_lazy(|| {
                format!("error while getting value for key {}", T::key_from(id))
            })?;

        value.map(RedisModel::deserialize_model).transpose()
    }

    /// Set a value in Redis.
    pub async fn set<T: RedisModel>(&self, value: &T) -> RedisResult<()> {
        let mut conn = self.conn().await?;

        if let Some(expires_after) = T::EXPIRES_AFTER {
            conn.set_ex(value.key(), value.serialize_model()?, expires_after)
                .await
                .report()
                .change_context(RedisClientError)
                .attach_printable_lazy(|| {
                    format!("failed to set expiration for key {}", value.key())
                })?;
        } else {
            conn.set(value.key(), value.serialize_model()?)
                .await
                .report()
                .change_context(RedisClientError)
                .attach_printable_lazy(|| format!("failed to set value for key {}", value.key()))?;
        }

        Ok(())
    }

    /// Run a `PING` command to check if Redis is connected.
    pub async fn ping(&self) -> RedisResult<()> {
        let mut conn = self.conn().await?;
        redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to run PING command")?;

        Ok(())
    }

    /// Get all the [`CachedChannel`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_channels(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedChannel>> {
        let guild = self.get::<CachedGuild>(&id).await?;

        if let Some(guild) = guild {
            let mut conn = self.conn().await?;
            let mut pipe = redis::pipe();

            for channel in &guild.channels {
                pipe.get(CachedChannel::key_from(channel));
            }

            let value: Vec<_> = pipe
                .query_async(&mut *conn)
                .await
                .report()
                .change_context(RedisClientError)
                .attach_printable_lazy(|| {
                    format!("failed to query channels for guild {}", guild.id)
                })
                .attach_printable_lazy(|| format!("queried channels: {:?}", &guild.channels))?;

            value
                .into_iter()
                .map(RedisModel::deserialize_model)
                .collect()
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all the [`CachedRole`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    pub async fn guild_roles(&self, id: Id<GuildMarker>) -> RedisResult<Vec<CachedRole>> {
        let guild = self.get::<CachedGuild>(&id).await?;

        if let Some(guild) = guild {
            let mut conn = self.conn().await?;
            let mut pipe = redis::pipe();

            for role in &guild.roles {
                pipe.get(CachedRole::key_from(&role));
            }

            let value: Vec<_> = pipe
                .query_async(&mut *conn)
                .await
                .report()
                .change_context(RedisClientError)
                .attach_printable_lazy(|| format!("failed to query roles for guild {}", guild.id))
                .attach_printable_lazy(|| format!("queried roles: {:?}", &guild.roles))?;

            value
                .into_iter()
                .map(RedisModel::deserialize_model)
                .collect()
        } else {
            Ok(Vec::new())
        }
    }

    /// Get a [`GuildPermissions`] for a given guild.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub async fn permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<GuildPermissions<'_>, Report<PermissionError>> {
        GuildPermissions::new(self, guild_id).await
    }

    /// Get the [`HttpClient`] got a given guild.
    pub fn http<'a>(&'a self, http: &'a HttpClient, guild_id: Id<GuildMarker>) -> CacheHttp<'a> {
        CacheHttp::new(self, http, guild_id)
    }
}

/// This trait is implemented by types representing a Redis model.
///
/// It provides methods to get the model key used in Redis, as well as methods
/// for serialization and deserialization.
pub trait RedisModel: Serialize + DeserializeOwned {
    /// Type used for the unique model identifier.
    type Id: ?Sized;

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
    /// The default implementation serialize the model in MessagePack using
    /// [`rmp_serde`] and compress it with [`zstd`].
    fn serialize_model(&self) -> RedisResult<Vec<u8>> {
        let serialized = rmp_serde::to_vec(self)
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to serialize data")?;

        zstd::encode_all(&*serialized, 0)
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to compress data")
    }

    /// Deserialize this model.
    ///
    /// The default implementation decompress the model with [`zstd`] and
    /// deserialize it from MessagePack with [`rmp_serde`].
    fn deserialize_model(value: Vec<u8>) -> RedisResult<Self> {
        let decoded = zstd::decode_all(&*value)
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to decompress data")?;

        rmp_serde::from_slice(&decoded)
            .report()
            .change_context(RedisClientError)
            .attach_printable("failed to deserialize data")
    }
}

/// Error occurred in [`RedisClient`] methods.
#[derive(Debug)]
pub struct RedisClientError;

impl fmt::Display for RedisClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("error while processing Redis request")
    }
}

impl Context for RedisClientError {}
