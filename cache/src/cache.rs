//! In-memory cache.
//!
//! This module expose the cache used by the gateway to store Discord data. This
//! cache only store necessary data, using models from the [`model`] module.
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
//! [`model`]: crate::model

use dashmap::{mapref::one::Ref, DashMap};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
    Id,
};

use crate::{
    model::{CachedChannel, CachedGuild, CachedRole},
    UpdateCache,
};

/// In-memory cache.
///
/// This type hold the cache using [`DashMap`]. Wrap it using an [`Arc`] to share
/// the cache between multiple threads.
///
/// The cache returns references ([`Ref`]) of cached values. While holding them,
/// the cache may be blocked if a resource need to be updated. These references
/// should not be hold between long-running tasks such as HTTP requests.
///
/// [`Arc`]: std::sync::Arc
#[derive(Debug)]
pub struct InMemoryCache {
    pub(crate) current_user: Id<UserMarker>,
    pub(crate) guilds: DashMap<Id<GuildMarker>, CachedGuild>,
    pub(crate) channels: DashMap<Id<ChannelMarker>, CachedChannel>,
    pub(crate) roles: DashMap<Id<RoleMarker>, CachedRole>,
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
    pub fn guild(&self, id: Id<GuildMarker>) -> Option<Ref<'_, Id<GuildMarker>, CachedGuild>> {
        self.guilds.get(&id)
    }

    /// Get all the [`CachedChannel`] of a guild.
    pub fn guild_channels(
        &self,
        id: Id<GuildMarker>,
    ) -> Option<Vec<Ref<'_, Id<ChannelMarker>, CachedChannel>>> {
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
    pub fn guild_roles(
        &self,
        id: Id<GuildMarker>,
    ) -> Option<Vec<Ref<'_, Id<RoleMarker>, CachedRole>>> {
        if let Some(guild) = self.guild(id) {
            let roles: Vec<_> = guild.roles.iter().filter_map(|id| self.role(*id)).collect();

            if !roles.is_empty() {
                return Some(roles);
            }
        }

        None
    }

    /// Get a [`CachedChannel`] by id.
    pub fn channel(
        &self,
        id: Id<ChannelMarker>,
    ) -> Option<Ref<'_, Id<ChannelMarker>, CachedChannel>> {
        self.channels.get(&id)
    }

    /// Get a [`CachedRole`] by id.
    pub fn role(&self, id: Id<RoleMarker>) -> Option<Ref<'_, Id<RoleMarker>, CachedRole>> {
        self.roles.get(&id)
    }
}
