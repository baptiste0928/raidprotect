//! Calculate the permissions for a member with information from the cache.
//!
//! This module allows to compute in-channel or guild permissions for a given
//! member using [`twilight_util::permission_calculator`].

use std::{
    cmp::Ordering,
    fmt::{self},
};

use error_stack::{Context, IntoReport, Report, ResultExt};
use twilight_model::{
    channel::ChannelType,
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};
use twilight_util::permission_calculator::PermissionCalculator;

use super::{
    model::{CachedChannel, CachedGuild, CachedRole},
    redis::{RedisClient, RedisModel},
};

/// Calculate the permissions for a given guild.
pub struct GuildPermissions<'a> {
    redis: &'a RedisClient,
    guild: CachedGuild,
}

impl<'a> GuildPermissions<'a> {
    /// Initialize [`GuildPermissions`] with from a guild.
    pub(crate) async fn new(
        redis: &'a RedisClient,
        guild_id: Id<GuildMarker>,
    ) -> Result<GuildPermissions<'a>, Report<PermissionError>> {
        if let Some(guild) = redis
            .get::<CachedGuild>(&guild_id)
            .await
            .change_context(PermissionError)?
        {
            Ok(Self { redis, guild })
        } else {
            Err(Report::new(PermissionError).attach_printable("guild not found in cache"))
        }
    }

    /// Compute permissions for a given guild member.
    pub async fn member(
        &self,
        member_id: Id<UserMarker>,
        member_roles: &[Id<RoleMarker>],
    ) -> Result<CachePermissions<'a>, Report<PermissionError>> {
        CachePermissions::new(self, member_id, member_roles).await
    }

    /// Compute permissions for the current bot member.
    pub async fn current_member(&self) -> Result<CachePermissions<'a>, Report<PermissionError>> {
        CachePermissions::current_member(self).await
    }
}

/// Calculate the permissions of a member with information from the cache.
pub struct CachePermissions<'a> {
    redis: &'a RedisClient,
    guild_id: Id<GuildMarker>,
    member_id: Id<UserMarker>,
    member_roles: MemberRoles,
    is_owner: bool,
}

impl<'a> CachePermissions<'a> {
    /// Initialize [`CachePermissions`] from a redis client.
    ///
    /// If the guild is not found in the cache, [`None`] is returned.
    pub(crate) async fn new(
        guild_permissions: &GuildPermissions<'a>,
        member_id: Id<UserMarker>,
        member_roles: &[Id<RoleMarker>],
    ) -> Result<CachePermissions<'a>, Report<PermissionError>> {
        let guild_id = guild_permissions.guild.id;
        let is_owner = member_id == guild_permissions.guild.owner_id;

        let member_roles =
            MemberRoles::query(guild_permissions.redis, guild_id, member_roles.iter()).await?;

        Ok(Self {
            redis: guild_permissions.redis,
            guild_id,
            member_id,
            member_roles,
            is_owner,
        })
    }

    /// Initialize [`CachePermissions`] for the bot current member.
    pub(crate) async fn current_member(
        guild_permissions: &GuildPermissions<'a>,
    ) -> Result<CachePermissions<'a>, Report<PermissionError>> {
        let member = guild_permissions.guild.current_member.as_ref().ok_or(
            Report::new(PermissionError).attach_printable("current member not found in cache"),
        )?;

        let guild_id = guild_permissions.guild.id;
        let is_owner = member.id == guild_permissions.guild.owner_id;

        let member_roles =
            MemberRoles::query(guild_permissions.redis, guild_id, member.roles.iter()).await?;

        Ok(Self {
            redis: guild_permissions.redis,
            guild_id,
            member_id: member.id,
            member_roles,
            is_owner,
        })
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

        // TODO: extract this into a function
        let everyone_role = self.member_roles.everyone.permissions;
        let member_roles = self
            .member_roles
            .roles
            .iter()
            .map(|role| (role.id, role.permissions))
            .collect::<Vec<_>>();

        let calculator =
            PermissionCalculator::new(self.guild_id, self.member_id, everyone_role, &member_roles);

        calculator.root()
    }

    /// Calculate the permissions of the user in a given channel.
    ///
    /// This method also return the [`ChannelType`] of the requested channel
    /// to handle the case where the channel is a thread.
    pub async fn channel(
        &self,
        channel: Id<ChannelMarker>,
    ) -> Result<(Permissions, ChannelType), Report<PermissionError>> {
        let mut channel = self
            .redis
            .get::<CachedChannel>(&channel)
            .await
            .change_context(PermissionError)?
            .ok_or(Report::new(PermissionError).attach_printable("channel not found in cache"))?;

        // If the channel is a thread, get the parent channel.
        if let CachedChannel::Thread(thread) = channel {
            channel = self
                .redis
                .get::<CachedChannel>(&thread.parent_id)
                .await
                .change_context(PermissionError)?
                .ok_or(
                    Report::new(PermissionError)
                        .attach_printable("parent channel not found in cache"),
                )?;
        }

        // TODO: extract this into a function
        let everyone_role = self.member_roles.everyone.permissions;
        let member_roles = self
            .member_roles
            .roles
            .iter()
            .map(|role| (role.id, role.permissions))
            .collect::<Vec<_>>();

        let calculator =
            PermissionCalculator::new(self.guild_id, self.member_id, everyone_role, &member_roles);

        let kind = channel.kind();
        let permissions = calculator.in_channel(kind, channel.permissions());

        Ok((permissions, kind))
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
    ) -> Result<MemberRoles, Report<PermissionError>> {
        let everyone_id = guild_id.cast();

        // Get user roles
        let mut pipe = redis::pipe();
        for role in member_roles.copied().chain([everyone_id]) {
            pipe.get(CachedRole::key_from(&role));
        }

        let mut conn = redis.conn().await.change_context(PermissionError)?;
        let result: Vec<_> = pipe
            .query_async(&mut *conn)
            .await
            .report()
            .change_context(PermissionError)
            .attach_printable("failed to query user roles")?;

        // Filter everyone role and other roles
        let mut everyone_role = None;
        let mut roles = Vec::new();

        for value in result {
            let role = CachedRole::deserialize_model(value).change_context(PermissionError)?;

            if role.id == everyone_id {
                everyone_role = Some(role);
            } else {
                roles.push(role)
            }
        }

        if let Some(everyone) = everyone_role {
            Ok(MemberRoles { everyone, roles })
        } else {
            Err(Report::new(PermissionError).attach_printable("everyone role not found in cache"))
        }
    }
}

/// Error occurred while computing permissions.
#[derive(Debug)]
pub struct PermissionError;

impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("failed to compute permissions")
    }
}

impl Context for PermissionError {}

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
