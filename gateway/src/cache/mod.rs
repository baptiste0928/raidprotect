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
use raidprotect_transport::cache::{Cache, CacheResult};
use twilight_model::id::{ChannelId, GuildId, RoleId, UserId};

/// In-memory cache.
#[derive(Debug)]
pub struct InMemoryCache {
    current_user: UserId,
    guilds: DashMap<GuildId, CachedGuild>,
    channels: DashMap<ChannelId, CachedChannel>,
    roles: DashMap<RoleId, CachedRole>,
}

impl InMemoryCache {
    /// Initialize a new [`InMemoryCache`].
    pub fn new(current_user: UserId) -> Self {
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
    pub fn guild(&self, id: GuildId) -> Option<&CachedGuild> {
        self.guilds.get(&id).map(|r| r.value())
    }

    /// Get all the [`CachedChannel`] of a guild.
    pub fn guild_channels(&self, id: GuildId) -> Option<Vec<&CachedChannel>> {
        if let Some(guild) = self.guild(id) {
            let channels: Vec<_> = guild
                .channels
                .iter()
                .filter_map(|id| self.channel(*id))
                .collect();

            if !channels.is_empty() {
                return Some(channels);
            }
        }

        None
    }

    /// Get all the [`CachedRole`] of a guild.
    pub fn guild_roles(&self, id: GuildId) -> Option<Vec<&CachedRole>> {
        if let Some(guild) = self.guild(id) {
            let roles: Vec<_> = guild.roles.iter().filter_map(|id| self.role(*id)).collect();

            if !roles.is_empty() {
                return Some(roles);
            }
        }

        None
    }

    /// Get a [`CachedChannel`] by id.
    pub fn channel(&self, id: ChannelId) -> Option<&CachedChannel> {
        self.channels.get(&id).map(|r| r.value())
    }

    /// Get a [`CachedRole`] by id.
    pub fn role(&self, id: RoleId) -> Option<&CachedRole> {
        self.roles.get(&id).map(|r| r.value())
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn guild(&self, id: GuildId) -> CacheResult<CachedGuild> {
        Ok(self.guild(id).cloned())
    }

    async fn channel(&self, id: ChannelId) -> CacheResult<CachedChannel> {
        Ok(self.channel(id).cloned())
    }

    async fn channels(&self, id: GuildId) -> CacheResult<Vec<CachedChannel>> {
        match self.guild_channels(id) {
            Some(channels) => Ok(Some(channels.into_iter().cloned().collect())),
            None => Ok(None),
        }
    }

    async fn role(&self, id: RoleId) -> CacheResult<CachedRole> {
        Ok(self.role(id).cloned())
    }

    async fn roles(&self, id: GuildId) -> CacheResult<Vec<CachedRole>> {
        match self.guild_roles(id) {
            Some(roles) => Ok(Some(roles.into_iter().cloned().collect())),
            None => Ok(None),
        }
    }
}
