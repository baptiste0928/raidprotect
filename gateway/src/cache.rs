//! In-memory cache.
//!
//! Cache is stored into [`DashMap`] in the memory. Methods are
//! available to update or query it.


use dashmap::{mapref::one::Ref, DashMap};
use raidprotect_model::cache::{CachedChannel, CachedGuild, CachedRole, CurrentMember};
use twilight_model::{
    id::{ChannelId, GuildId, RoleId},
};

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
    pub fn get_guild(&self, id: GuildId) -> Option<Ref<'_, GuildId, CachedGuild>> {
        self.guilds.get(&id)
    }

    /// Get a [`CachedChannel`] by id.
    pub fn get_channel(&self, id: ChannelId) -> Option<Ref<'_, ChannelId, CachedChannel>> {
        self.channels.get(&id)
    }

    /// Get a [`CachedRole`] by id.
    pub fn get_role(&self, id: RoleId) -> Option<Ref<'_, RoleId, CachedRole>> {
        self.roles.get(&id)
    }

    /// Insert a [`CachedGuild`] in the cache.
    ///
    /// The old value is returned if existing.
    pub fn insert_guild(&self, guild: CachedGuild) -> Option<CachedGuild> {
        self.guilds.insert(guild.id, guild)
    }

    /// Insert a [`CurrentMember`] in the cache.
    pub fn insert_current_member(&self, id: GuildId, member: CurrentMember) {
        if let Some(mut guild) = self.guilds.get_mut(&id) {
            guild.current_member = Some(member)
        }
    }

    /// Insert a [`CachedChannel`] in the cache.
    ///
    /// The old value is returned if existing.
    pub fn insert_channel(&self, channel: CachedChannel) -> Option<CachedChannel> {
        if let Some(mut guild) = self.guilds.get_mut(&channel.guild_id()) {
            guild.channels.insert(channel.id());
        }

        self.channels.insert(channel.id(), channel)
    }

    /// Insert a [`CachedRole`] in the cache.
    ///
    /// The old value is returned if existing.
    pub fn insert_role(&self, role: CachedRole) -> Option<CachedRole> {
        if let Some(mut guild) = self.guilds.get_mut(&role.guild_id) {
            guild.roles.insert(role.id);
        }

        self.roles.insert(role.id, role)
    }

    /// Remove a guild and associated items from the cache.
    ///
    /// The removed value is returned if existing.
    pub fn remove_guild(&self, guild: GuildId) -> Option<CachedGuild> {
        if let Some((_, guild)) = self.guilds.remove(&guild) {
            for channel in &guild.channels {
                self.channels.remove(channel);
            }

            for role in &guild.roles {
                self.roles.remove(role);
            }

            Some(guild)
        } else {
            None
        }
    }

    /// Remove a channel from the cache.
    ///
    /// The removed value is returned if existing.
    pub fn remove_channel(&self, guild: GuildId, channel: ChannelId) -> Option<CachedChannel> {
        if let Some(mut guild) = self.guilds.get_mut(&guild) {
            guild.channels.remove(&channel);
        }

        self.channels.remove(&channel).map(|(_, value)| value)
    }

    /// Remove a role from the cache.
    ///
    /// The removed value is returned if existing.
    pub fn remove_role(&self, guild: GuildId, role: RoleId) -> Option<CachedRole> {
        if let Some(mut guild) = self.guilds.get_mut(&guild) {
            guild.roles.remove(&role);
        }

        self.roles.remove(&role).map(|(_, value)| value)
    }
}
