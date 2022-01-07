//! In-memory cache.
//!
//! Cache is stored into [`DashMap`] in the memory. Methods are
//! available to update or query it.

use dashmap::{mapref::one::Ref, DashMap};
use raidprotect_model::cache::{CachedChannel, CachedGuild, CachedRole};
use raidprotect_transport::cache::{Cache, CacheResult};
use remoc::rtc;
use twilight_model::id::{ChannelId, GuildId, RoleId};

/// In-memory cache.
///
/// This type holds the gateway cache and expose methods
/// to query or update it.
#[derive(Debug, Default)]
pub struct InMemoryCache {
    guilds: DashMap<GuildId, CachedGuild>,
    channels: DashMap<ChannelId, CachedChannel>,
    roles: DashMap<RoleId, CachedRole>,
}

impl InMemoryCache {
    /// Initialize a new [`InMemoryCache`].
    fn new() -> Self {
        Self::default()
    }

    /// Get a [`CachedGuild`] by id.
    pub fn guild(&self, id: GuildId) -> Option<Ref<'_, GuildId, CachedGuild>> {
        self.guilds.get(&id)
    }

    /// Get a [`CachedChannel`] by id.
    pub fn channel(&self, id: ChannelId) -> Option<Ref<'_, ChannelId, CachedChannel>> {
        self.channels.get(&id)
    }

    /// Get a [`CachedRole`] by id.
    pub fn role(&self, id: RoleId) -> Option<Ref<'_, RoleId, CachedRole>> {
        self.roles.get(&id)
    }
}

#[rtc::async_trait]
impl Cache for InMemoryCache {
    async fn guild(&self, id: GuildId) -> CacheResult<CachedGuild> {
        Ok(self.guild(id).as_deref().cloned())
    }

    async fn channel(&self, id: ChannelId) -> CacheResult<CachedChannel> {
        Ok(self.channel(id).as_deref().cloned())
    }

    async fn channels(&self, id: GuildId) -> CacheResult<Vec<CachedChannel>> {
        if let Some(guild) = self.guild(id) {
            let channels: Vec<CachedChannel> = guild
                .channels
                .iter()
                .filter_map(|id| self.channel(*id).as_deref().cloned())
                .collect();

            if !channels.is_empty() {
                return Ok(Some(channels));
            }
        }

        Ok(None)
    }

    async fn role(&self, id: RoleId) -> CacheResult<CachedRole> {
        Ok(self.role(id).as_deref().cloned())
    }

    async fn roles(&self, id: GuildId) -> CacheResult<Vec<CachedRole>> {
        if let Some(guild) = self.guild(id) {
            let roles: Vec<CachedRole> = guild
                .roles
                .iter()
                .filter_map(|id| self.role(*id).as_deref().cloned())
                .collect();

            if !roles.is_empty() {
                return Ok(Some(roles));
            }
        }

        Ok(None)
    }
}
