//! Calculate the permissions for a member with information from the cache.
//!
//! This module allows to compute in-channel or guild permissions for a given
//! member using [`twilight_util::permission_calculator`].

use std::cmp::Ordering;

use twilight_model::{
    guild::Permissions,
    id::{
        marker::{GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};
use twilight_util::permission_calculator::PermissionCalculator;

use crate::{
    model::{CachedGuild, CachedRole},
    redis::{RedisClient, RedisModel, RedisResult},
};

/// Calculate the permissions of a member with information from the cache.
pub struct CachePermissions {
    guild_id: Id<GuildMarker>,
    user_id: Id<UserMarker>,
    member_roles: MemberRoles,
    is_owner: bool,
}

impl CachePermissions {
    /// Initialize [`CachePermissions`] from a redis client.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub(crate) async fn new(
        redis: &RedisClient,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        member_roles: &[Id<RoleMarker>],
    ) -> RedisResult<Option<Self>> {
        if let Some(guild) = dbg!(redis.get::<CachedGuild>(&guild_id).await?) {
            let is_owner = user_id == guild.owner_id;

            if let Some(member_roles) =
                MemberRoles::query(redis, guild_id, member_roles.iter()).await?
            {
                return Ok(Some(Self {
                    guild_id,
                    user_id,
                    member_roles,
                    is_owner,
                }));
            }
        }

        Ok(None)
    }

    /// Initialize [`CachePermissions`] for the bot current member.
    pub(crate) async fn current_member(
        redis: &RedisClient,
        guild_id: Id<GuildMarker>,
    ) -> RedisResult<Option<Self>> {
        if let Some(guild) = redis.get::<CachedGuild>(&guild_id).await? {
            if let Some(current_member) = guild.current_member {
                let is_owner = current_member.id == guild.owner_id;

                if let Some(member_roles) =
                    MemberRoles::query(redis, guild_id, current_member.roles.iter()).await?
                {
                    return Ok(Some(Self {
                        guild_id,
                        user_id: current_member.id,
                        member_roles,
                        is_owner,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Checks if a user is the owner of a guild.
    pub fn is_owner(&self) -> bool {
        self.is_owner
    }

    /// Returns the highest role of a user.
    pub fn highest_role(&self) -> RoleOrdering {
        if self.member_roles.roles.is_empty() {
            RoleOrdering::from_cached(&self.member_roles.everyone)
        } else {
            let mut roles: Vec<_> = self
                .member_roles
                .roles
                .iter()
                .map(RoleOrdering::from_cached)
                .collect();
            roles.sort();

            *roles.last().unwrap() // SAFETY: roles is not empty
        }
    }

    /// Calculate the permissions of the user in the guild.
    pub fn guild(&self) -> Permissions {
        // Owners have all permissions
        if self.is_owner {
            return Permissions::all();
        }

        let everyone_role = self.member_roles.everyone.permissions;
        let member_roles = self
            .member_roles
            .roles
            .iter()
            .map(|role| (role.id, role.permissions))
            .collect::<Vec<_>>();

        let calculator = PermissionCalculator::new(
            self.guild_id,
            self.user_id,
            everyone_role,
            member_roles.as_slice(),
        );

        calculator.root()
    }
}

/// List of resolved roles of a member.
struct MemberRoles {
    /// Everyone role
    pub everyone: CachedRole,
    /// List of roles of the user
    pub roles: Vec<CachedRole>,
}

impl MemberRoles {
    /// Query roles of a member in the cache.
    async fn query(
        redis: &RedisClient,
        guild_id: Id<GuildMarker>,
        member_roles: impl Iterator<Item = &Id<RoleMarker>>,
    ) -> RedisResult<Option<MemberRoles>> {
        // Get user roles
        let mut pipe = redis::pipe();
        for role in member_roles {
            pipe.get(CachedRole::key_from(role));
        }

        let mut conn = redis.conn().await?;
        let result: Vec<_> = pipe.query_async(&mut *conn).await?;

        // Filter everyone role and other roles
        let everyone_id = guild_id.cast();
        let mut everyone_role = None;
        let mut roles = Vec::new();

        for value in result {
            let role = CachedRole::deserialize_model(value)?;

            if role.id == everyone_id {
                everyone_role = Some(role);
            } else {
                roles.push(role)
            }
        }

        if let Some(everyone) = everyone_role {
            Ok(Some(MemberRoles { everyone, roles }))
        } else {
            Ok(None)
        }
    }
}

/// Compares the position of two roles.
///
/// This type is used to compare positions of two different roles, using the
/// [`Ord`] trait.
///
/// According to [twilight-model documentation]:
///
/// > Roles are primarily ordered by their position in descending order.
/// > For example, a role with a position of 17 is considered a higher role than
/// > one with a position of 12.
/// >
/// > Discord does not guarantee that role positions are positive, unique, or
/// > contiguous. When two or more roles have the same position then the order
/// > is based on the roles’ IDs in ascending order. For example, given two roles
/// > with positions of 10 then a role with an ID of 1 would be considered a
/// > higher role than one with an ID of 20.
///
/// [twilight-model documentation]: https://docs.rs/twilight-model/0.10.2/twilight_model/guild/struct.Role.html#impl-Ord
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleOrdering {
    id: Id<RoleMarker>,
    position: i64,
}

impl RoleOrdering {
    /// Initialize a new [`RoleOrdering`] from a [`CachedRole`].
    pub(crate) fn from_cached(role: &CachedRole) -> Self {
        Self {
            id: role.id,
            position: role.position,
        }
    }
}

impl Ord for RoleOrdering {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position
            .cmp(&other.position)
            .then(self.id.get().cmp(&other.id.get()))
    }
}

impl PartialOrd for RoleOrdering {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_roles() {
        let role_a = RoleOrdering {
            id: Id::new(123),
            position: 12,
        };

        let role_b = RoleOrdering {
            id: Id::new(456),
            position: 13,
        };

        assert_eq!(Ordering::Less, role_a.cmp(&role_b));
        assert_eq!(Ordering::Greater, role_b.cmp(&role_a));
        assert_eq!(Ordering::Equal, role_a.cmp(&role_a));
        assert_eq!(Ordering::Equal, role_b.cmp(&role_b));
    }

    #[test]
    fn cmp_roles_same_position() {
        let role_a = RoleOrdering {
            id: Id::new(123),
            position: 12,
        };

        let role_b = RoleOrdering {
            id: Id::new(456),
            position: 12,
        };

        assert_eq!(Ordering::Less, role_a.cmp(&role_b));
        assert_eq!(Ordering::Greater, role_b.cmp(&role_a));
        assert_eq!(Ordering::Equal, role_a.cmp(&role_a));
        assert_eq!(Ordering::Equal, role_b.cmp(&role_b));
    }
}
