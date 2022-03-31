//! Calculate the permissions for a member with information from the cache.
//!
//! This module allows to compute in-channel or guild permissions for a given
//! member using [`twilight_util::permission_calculator`].

use twilight_model::{
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};
use twilight_util::permission_calculator::PermissionCalculator;

use crate::InMemoryCache;

/// Calculate the permissions of a member with information from the cache.
#[derive(Debug, Clone, Copy)]
pub struct CachePermissions<'a> {
    cache: &'a InMemoryCache,
}

impl<'a> CachePermissions<'a> {
    /// Initialize [`CachePermissions`] from a cache reference.
    pub fn new(cache: &'a InMemoryCache) -> Self {
        Self { cache }
    }

    /// Calculate the permissions of a member in a guild channel.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub fn in_channel(
        &self,
        user_id: Id<UserMarker>,
        roles: &[Id<RoleMarker>],
        channel_id: Id<ChannelMarker>,
    ) -> Option<Permissions> {
        let channel = self.cache.channel(channel_id)?;
        let guild = self.cache.guild(channel.guild_id())?;

        // Owners have all permissions
        if user_id == guild.owner_id {
            return Some(Permissions::all());
        }

        // Get permissions of user roles
        let guild_roles = self.cache.guild_roles(guild.id)?;
        let member_roles = roles
            .iter()
            .filter(|role_id| **role_id != guild.id.cast()) // Ignore everyone role
            .map(|role_id| {
                let permissions = guild_roles
                    .iter()
                    .find(|role| role.id == *role_id)?
                    .permissions;

                Some((*role_id, permissions))
            })
            .collect::<Option<Vec<_>>>()?;

        let everyone_role = guild_roles
            .iter()
            .find(|role| role.id == guild.id.cast())?
            .permissions;

        // Get channel permissions overwrite

        // Calculate permissions
        let calculator =
            PermissionCalculator::new(guild.id, user_id, everyone_role, member_roles.as_slice());

        todo!()
    }

    /// Calculate the permissions of a user in a guild.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub fn guild(
        &self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        roles: &[Id<RoleMarker>],
    ) -> Option<Permissions> {
        let guild = self.cache.guild(guild_id)?;

        // Owners have all permissions
        if user_id == guild.owner_id {
            return Some(Permissions::all());
        }

        let everyone_role = self.cache.role(guild.id.cast())?.permissions;
        let user_roles = roles
            .iter()
            .filter(|id| **id != guild.id.cast())  // Remove everyone role
            .filter_map(|id| self.cache.role(*id))
            .collect::<Vec<_>>();


        todo!()
    }
}
