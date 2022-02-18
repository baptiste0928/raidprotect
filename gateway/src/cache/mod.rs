//! In-memory cache.
//!
//! This module expose the cache used by the gateway to store Discord data. This
//! cache only store necessary data, using models from the [`model::cache`]
//! module.
//!
//! Internally, data is stored in a [`DashMap`] to allow efficient concurrent
//! access. Methods are exposed to query the cache.
//!
//! The cache is filled using data from incoming events, using the [`UpdateCache`]
//! trait.
//!
//! ## Processed event types
//! | Cached data           | Event types                                                       |
//! |-----------------------|-------------------------------------------------------------------|
//! | Guilds                | `GuildCreate`, `GuildUpdate`, `GuildDelete`                       |
//! | Channels (guild-only) | `ChannelCreate`, `ChannelUpdate`, `ChannelUpdate` (+ thread ones) |
//! | Roles                 | `RoleCreate`, `RoleUpdate`, `RoleDelete`                          |
//! | Current user member   | `MemberAdd`, `MemberUpdate`                                       |
//!
//! [`model::cache`]: raidprotect_model::cache

mod event;
mod resource;

pub use event::UpdateCache;

use async_trait::async_trait;
use dashmap::DashMap;
use raidprotect_model::cache::{CachedChannel, CachedGuild, CachedRole};
use raidprotect_transport::cache::{Cache, CacheError};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
    Id,
};

/// In-memory cache.
#[derive(Debug)]
pub struct InMemoryCache {
    current_user: Id<UserMarker>,
    guilds: DashMap<Id<GuildMarker>, CachedGuild>,
    channels: DashMap<Id<ChannelMarker>, CachedChannel>,
    roles: DashMap<Id<RoleMarker>, CachedRole>,
}

impl InMemoryCache {
    /// Initialize a new [`InMemoryCache`].
    pub fn new(current_user: Id<UserMarker>) -> Self {
        Self {
            current_user,
            guilds: DashMap::new(),
            channels: DashMap::new(),
            roles: DashMap::new(),
        }
    }

    /// Update the cache based on event data.
    pub fn update<T>(&self, item: &T) -> T::Output
    where
        T: UpdateCache,
    {
        item.update(self)
    }

    /// Get a [`CachedGuild`] by id.
    pub fn guild(&self, id: Id<GuildMarker>) -> Option<CachedGuild> {
        self.guilds.get(&id).as_deref().cloned()
    }

    /// Get all the [`CachedChannel`] of a guild.
    pub fn guild_channels(&self, id: Id<GuildMarker>) -> Option<Vec<CachedChannel>> {
        if let Some(guild) = self.guild(id) {
            let channels: Vec<_> = guild
                .channels
                .into_iter()
                .filter_map(|id| self.channel(id))
                .collect();

            if !channels.is_empty() {
                return Some(channels);
            }
        }

        None
    }

    /// Get all the [`CachedRole`] of a guild.
    pub fn guild_roles(&self, id: Id<GuildMarker>) -> Option<Vec<CachedRole>> {
        if let Some(guild) = self.guild(id) {
            let roles: Vec<_> = guild
                .roles
                .into_iter()
                .filter_map(|id| self.role(id))
                .collect();

            if !roles.is_empty() {
                return Some(roles);
            }
        }

        None
    }

    /// Get a [`CachedChannel`] by id.
    pub fn channel(&self, id: Id<ChannelMarker>) -> Option<CachedChannel> {
        self.channels.get(&id).as_deref().cloned()
    }

    /// Get a [`CachedRole`] by id.
    pub fn role(&self, id: Id<RoleMarker>) -> Option<CachedRole> {
        self.roles.get(&id).as_deref().cloned()
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn guild(&self, id: Id<GuildMarker>) -> Result<Option<CachedGuild>, CacheError> {
        Ok(self.guild(id))
    }

    async fn channel(&self, id: Id<ChannelMarker>) -> Result<Option<CachedChannel>, CacheError> {
        Ok(self.channel(id))
    }

    async fn channels(
        &self,
        id: Id<GuildMarker>,
    ) -> Result<Option<Vec<CachedChannel>>, CacheError> {
        Ok(self.guild_channels(id))
    }

    async fn role(&self, id: Id<RoleMarker>) -> Result<Option<CachedRole>, CacheError> {
        Ok(self.role(id))
    }

    async fn roles(&self, id: Id<GuildMarker>) -> Result<Option<Vec<CachedRole>>, CacheError> {
        Ok(self.guild_roles(id))
    }
}
