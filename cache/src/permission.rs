//! Calculate the permissions for a member with information from the cache.
//!
//! This module allows to compute in-channel or guild permissions for a given
//! member using [`twilight_util::permission_calculator`].

use twilight_model::{
    guild::Permissions,
    id::{
        marker::{GuildMarker, RoleMarker, UserMarker},
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

    /// Calculate the permissions of a user in a guild.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub fn guild(
        &self,
        user_id: Id<UserMarker>,
        guild_id: Id<GuildMarker>,
        member_roles: &[Id<RoleMarker>],
    ) -> Option<Permissions> {
        let guild = self.cache.guild(guild_id)?;

        // Owners have all permissions
        if user_id == guild.owner_id {
            return Some(Permissions::all());
        }

        // Get member roles from cache
        let everyone_role = self.cache.role(guild.id.cast())?.permissions;
        let member_roles = member_roles
            .iter()
            .filter(|id| **id != guild.id.cast()) // Remove everyone role
            .map(|id| {
                let role = self.cache.role(*id)?;

                Some((role.id, role.permissions))
            })
            .collect::<Option<Vec<_>>>()?;

        // Calculate permissions
        let calculator =
            PermissionCalculator::new(guild_id, user_id, everyone_role, member_roles.as_slice());

        Some(calculator.root())
    }
}
