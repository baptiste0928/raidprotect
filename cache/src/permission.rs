//! Calculate the permissions for a member with information from the cache.
//!
//! This module allows to compute in-channel or guild permissions for a given
//! member using [`twilight_util::permission_calculator`].

use dashmap::mapref::one::Ref;
use twilight_model::{
    guild::Permissions,
    id::{
        marker::{GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};
use twilight_util::permission_calculator::PermissionCalculator;

use crate::{model::CachedRole, InMemoryCache};

/// Calculate the permissions of a member with information from the cache.
#[derive(Debug, Clone, Copy)]
pub struct CachePermissions<'cache> {
    cache: &'cache InMemoryCache,
    guild_id: Id<GuildMarker>,
}

impl<'cache> CachePermissions<'cache> {
    /// Initialize [`CachePermissions`] from a cache reference.
    pub fn new(cache: &'cache InMemoryCache, guild_id: Id<GuildMarker>) -> Self {
        Self { cache, guild_id }
    }

    /// Checks if a user is the owner of a guild.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub fn is_owner(&self, user_id: Id<UserMarker>) -> Option<bool> {
        let guild = self.cache.guild(self.guild_id)?;

        Some(user_id == guild.owner_id)
    }

    /// Calculate the permissions of a user in a guild.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub fn guild(
        &self,
        user_id: Id<UserMarker>,
        member_roles: &[Id<RoleMarker>],
    ) -> Option<Permissions> {
        let guild = self.cache.guild(self.guild_id)?;

        // Owners have all permissions
        if user_id == guild.owner_id {
            return Some(Permissions::all());
        }

        let cached_roles = self.member_roles(member_roles)?;
        let everyone_role = cached_roles.everyone.permissions;
        let member_roles = cached_roles
            .roles
            .iter()
            .map(|role| (role.id, role.permissions))
            .collect::<Vec<_>>();

        let calculator = PermissionCalculator::new(
            self.guild_id,
            user_id,
            everyone_role,
            member_roles.as_slice(),
        );

        Some(calculator.root())
    }

    /// Fetch roles of a member in the cache.
    fn member_roles(&self, member_roles: &[Id<RoleMarker>]) -> Option<MemberRoles> {
        let everyone_id = self.guild_id.cast();

        let everyone = self.cache.role(everyone_id)?;
        let roles = member_roles
            .iter()
            .filter(|id| **id != everyone_id) // Remove everyone role
            .map(|id| self.cache.role(*id))
            .collect::<Option<Vec<_>>>()?;

        Some(MemberRoles { everyone, roles })
    }
}

/// Reference to a [`CachedRole`].
type CachedRoleRef<'cache> = Ref<'cache, Id<RoleMarker>, CachedRole>;

/// List of resolved roles of a member.
struct MemberRoles<'cache> {
    /// Everyone role
    pub everyone: CachedRoleRef<'cache>,

    /// List of roles of the user
    pub roles: Vec<CachedRoleRef<'cache>>,
}
