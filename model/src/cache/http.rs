//! Discord HTTP client using information from the cache.
//!
//! This module exports wrapper around twilight's HTTP client that use the cache
//! data to check permissions before making requests.

use anyhow::anyhow;
use twilight_http::{
    request::{
        channel::{message::CreateMessage, UpdateChannelPermission},
        guild::{
            member::{AddRoleToMember, RemoveMember},
            CreateGuildChannel,
        },
    },
    Client as HttpClient,
};
use twilight_model::{
    guild::Permissions,
    http::permission_overwrite::PermissionOverwrite,
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};

use super::{model::CachedRole, permission::RoleOrdering, redis::RedisClient};

/// HTTP client with permission checks.
#[derive(Debug)]
pub struct CacheHttp<'a> {
    redis: &'a RedisClient,
    http: &'a HttpClient,
    guild_id: Id<GuildMarker>,
}

impl<'a> CacheHttp<'a> {
    /// Initialize a new [`CacheHttp`].
    pub(crate) fn new(
        redis: &'a RedisClient,
        http: &'a HttpClient,
        guild_id: Id<GuildMarker>,
    ) -> Self {
        Self {
            redis,
            http,
            guild_id,
        }
    }

    /// Send a message to a channel.
    ///
    /// This method ensures that the bot has the [`SEND_MESSAGES`],
    /// [`SEND_MESSAGES_IN_THREADS`], [`USE_EXTERNAL_EMOJIS`] and [`EMBED_LINKS`]
    /// permissions before executing the request.
    ///
    /// [`SEND_MESSAGES`]: Permissions::SEND_MESSAGES
    /// [`SEND_MESSAGES_IN_THREADS`]: Permissions::SEND_MESSAGES_IN_THREADS
    /// [`USE_EXTERNAL_EMOJIS`]: Permissions::USE_EXTERNAL_EMOJIS
    /// [`EMBED_LINKS`]: Permissions::EMBED_LINKS
    pub async fn create_message(
        &self,
        channel: Id<ChannelMarker>,
    ) -> Result<CreateMessage<'a>, anyhow::Error> {
        let permissions = self.redis.permissions(self.guild_id).await?;
        let (permissions, kind) = permissions.current_member().await?.channel(channel).await?;

        let send_messages = if kind.is_thread() {
            Permissions::SEND_MESSAGES
        } else {
            Permissions::SEND_MESSAGES_IN_THREADS
        };

        if !permissions
            .contains(send_messages | Permissions::USE_EXTERNAL_EMOJIS | Permissions::EMBED_LINKS)
        {
            return Err(anyhow!("missing permissions to send message"));
        }

        Ok(self.http.create_message(channel))
    }

    /// Create a new guild channel.
    ///
    /// This method ensure that the bot has the [`MANAGE_CHANNELS`] permission.
    ///
    /// [`MANAGE_CHANNELS`]: Permissions::MANAGE_CHANNELS
    pub async fn create_guild_channel(
        &'a self,
        name: &'a str,
    ) -> Result<CreateGuildChannel<'a>, anyhow::Error> {
        let permissions = self
            .redis
            .permissions(self.guild_id)
            .await?
            .current_member()
            .await?
            .guild();

        if !permissions.contains(Permissions::MANAGE_CHANNELS) {
            return Err(anyhow!("missing permissions to create channel"));
        }

        Ok(self.http.create_guild_channel(self.guild_id, name)?)
    }

    /// Update a channel's permission overwrite.
    ///
    /// This method ensures that the bot has the [`MANAGE_ROLES`] and
    /// [`MANAGE_CHANNELS`] permission.
    ///
    /// [`MANAGE_ROLES`]: Permissions::MANAGE_ROLES
    /// [`MANAGE_CHANNELS`]: Permissions::MANAGE_CHANNELS
    pub async fn update_channel_permission(
        &'a self,
        channel_id: Id<ChannelMarker>,
        permission_overwrite: &'a PermissionOverwrite,
    ) -> Result<UpdateChannelPermission<'a>, anyhow::Error> {
        let permissions = self
            .redis
            .permissions(self.guild_id)
            .await?
            .current_member()
            .await?
            .guild();

        if !permissions.contains(Permissions::MANAGE_ROLES | Permissions::MANAGE_CHANNELS) {
            return Err(anyhow!("missing permissions to update channel permissions"));
        }

        Ok(self
            .http
            .update_channel_permission(channel_id, permission_overwrite))
    }

    /// Add a role to a member.
    ///
    /// This method ensures that the bot has the [`MANAGE_ROLES`] permission and
    /// the role to give is lower than the bot's highest role.
    ///
    /// [`MANAGE_ROLES`]: Permissions::MANAGE_ROLES
    pub async fn add_guild_member_role(
        &'a self,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) -> Result<AddRoleToMember<'a>, anyhow::Error> {
        let permissions = self
            .redis
            .permissions(self.guild_id)
            .await?
            .current_member()
            .await?;

        if !permissions.guild().contains(Permissions::MANAGE_ROLES) {
            return Err(anyhow!("missing permissions to add role to member"));
        }

        let role = match self.redis.get::<CachedRole>(&role_id).await? {
            Some(role) => role,
            None => return Err(anyhow!("role to add not found")),
        };

        if RoleOrdering::from(&role) >= permissions.highest_role() {
            return Err(anyhow!(
                "role to add role is higher than bot's highest role"
            ));
        }

        Ok(self
            .http
            .add_guild_member_role(self.guild_id, user_id, role_id))
    }

    /// Kick a user from a guild.
    ///
    /// This method ensures that the bot has the [`KICK_MEMBERS`] permission. It
    /// does not check for the role hierarchy.
    ///
    /// [`KICK_MEMBERS`]: Permissions::KICK_MEMBERS
    pub async fn remove_guild_member(
        &'a self,
        user_id: Id<UserMarker>,
    ) -> Result<RemoveMember<'a>, anyhow::Error> {
        let permissions = self
            .redis
            .permissions(self.guild_id)
            .await?
            .current_member()
            .await?;

        if !permissions.guild().contains(Permissions::KICK_MEMBERS) {
            return Err(anyhow!("missing permissions to kick member"));
        }

        Ok(self.http.remove_guild_member(self.guild_id, user_id))
    }
}
