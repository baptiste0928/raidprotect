//! Methods to query Discord objects in the cache for [`CacheClient`].

use tracing::{instrument, trace};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::GuildMarker, Id};

use super::{
    http::CacheHttp, permission::GuildPermissions, CachedChannel, CachedGuild, CachedRole,
};
use crate::cache::{CacheClient, RedisModel};

impl CacheClient {
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

    /// Get the [`CacheHttp`] client for a given guild.
    pub fn http<'a>(&'a self, http: &'a HttpClient, guild_id: Id<GuildMarker>) -> CacheHttp<'a> {
        CacheHttp::new(self, http, guild_id)
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
}
