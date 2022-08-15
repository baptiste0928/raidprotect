//! Redis client.
//!
//! This module expose the [`RedisClient] type used to access the cache stored
//! in Redis.

use std::{fmt::Debug, time::Duration};

use anyhow::Context;
use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{instrument, trace};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::GuildMarker, Id};

use super::{
    http::CacheHttp,
    model::{CachedChannel, CachedGuild, CachedRole},
    permission::GuildPermissions,
};

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
    pub async fn new(uri: &str) -> Result<Self, anyhow::Error> {
        let manager =
            RedisConnectionManager::new(uri).context("failed to initialize connection manager")?;

        let pool = Pool::builder()
            .connection_timeout(Duration::from_secs(2))
            .build(manager)
            .await
            .context("failed to initialize connection pool")?;

        Ok(Self { pool })
    }

    /// Get a new connection from the connection pool
    pub async fn conn(&self) -> Result<RedisConnection<'_>, anyhow::Error> {
        Ok(self.pool.get().await?)
    }

    /// Get a value from Redis.
    #[instrument(skip(self))]
    pub async fn get<T: RedisModel>(&self, id: &T::Id) -> Result<Option<T>, anyhow::Error> {
        let mut conn = self.conn().await?;
        let key = T::key_from(id);

        trace!("getting value for key {}", key);
        let value: Option<_> = conn.get(&key).await?;

        value.map(RedisModel::deserialize_model).transpose()
    }

    /// Set a value in Redis.
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

    /// Run a `PING` command to check if Redis is connected.
    pub async fn ping(&self) -> Result<(), anyhow::Error> {
        let mut conn = self.conn().await?;
        redis::cmd("PING").query_async(&mut *conn).await?;

        Ok(())
    }

    /// Get all the [`CachedChannel`] of a guild.
    ///
    /// If the guild is not cached, an empty [`Vec`] is returned.
    #[instrument(skip(self))]
    pub async fn guild_channels(
        &self,
        id: Id<GuildMarker>,
    ) -> Result<Vec<CachedChannel>, anyhow::Error> {
        let guild = self.get::<CachedGuild>(&id).await?;

        if let Some(guild) = guild {
            let mut conn = self.conn().await?;
            let mut pipe = redis::pipe();

            trace!(
                channels = ?guild.channels,
                "querying channels for guild {}",
                id
            );
            for channel in &guild.channels {
                pipe.get(CachedChannel::key_from(channel));
            }

            let value: Vec<_> = pipe.query_async(&mut *conn).await?;

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
    #[instrument(skip(self))]
    pub async fn guild_roles(&self, id: Id<GuildMarker>) -> Result<Vec<CachedRole>, anyhow::Error> {
        let guild = self.get::<CachedGuild>(&id).await?;

        if let Some(guild) = guild {
            let mut conn = self.conn().await?;
            let mut pipe = redis::pipe();

            trace!(roles = ?guild.roles, "querying roles for guild {}", id);
            for role in &guild.roles {
                pipe.get(CachedRole::key_from(role));
            }

            let value: Vec<_> = pipe.query_async(&mut *conn).await?;

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
    #[instrument(skip(self))]
    pub async fn permissions(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<GuildPermissions<'_>, anyhow::Error> {
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
pub trait RedisModel: Debug + Serialize + DeserializeOwned {
    /// Type used for the unique model identifier.
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
