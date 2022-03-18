//! Insert Discord models into the cache.
//!
//! This module contains functions to update the cache from Discord models. The
//! various functions convert the value into the one used in the cache.

use std::collections::HashSet;

use thiserror::Error;
use tracing::error;
use twilight_model::{
    channel::{Channel, ChannelType},
    guild::{Guild, Role},
    id::{marker::GuildMarker, Id},
};

use crate::{
    cache::InMemoryCache,
    model::{
        CachedCategoryChannel, CachedChannel, CachedGuild, CachedRole, CachedTextChannel,
        CachedThread, CurrentMember,
    },
};

pub fn cache_guild(cache: &InMemoryCache, guild: &Guild) -> Option<CachedGuild> {
    // Insert channels and roles into the cache.
    let mut channels = HashSet::with_capacity(guild.channels.len());
    let mut roles = HashSet::with_capacity(guild.roles.len());

    for channel in &guild.channels {
        if CachedChannel::is_cached(channel.kind) {
            match cache_guild_channel(cache, channel) {
                Ok(_) => {
                    channels.insert(channel.id);
                }
                Err(error) => {
                    error!(error = %error, "failed to cache guild channel");
                }
            };
        }
    }

    for role in &guild.roles {
        cache_role(cache, role, guild.id);

        roles.insert(role.id);
    }

    // Find the bot current member.
    let current_member = guild
        .members
        .iter()
        .find(|m| m.user.id == cache.current_user)
        .map(|member| CurrentMember {
            id: member.user.id,
            communication_disabled_until: member.communication_disabled_until,
            roles: member.roles.iter().copied().collect(),
        });

    // Insert the guild into the cache.
    let cached = CachedGuild {
        id: guild.id,
        unavailable: guild.unavailable,
        name: guild.name.clone(),
        icon: guild.icon,
        owner_id: guild.owner_id,
        current_member,
        roles,
        channels,
    };

    cache.guilds.insert(guild.id, cached)
}

pub fn cache_role(
    cache: &InMemoryCache,
    role: &Role,
    guild_id: Id<GuildMarker>,
) -> Option<CachedRole> {
    let cached = CachedRole {
        id: role.id,
        guild_id,
        name: role.name.clone(),
        color: role.color,
        icon: role.icon,
        unicode_emoji: role.unicode_emoji.clone(),
        position: role.position,
        permissions: role.permissions,
        managed: role.managed,
    };

    cache.roles.insert(role.id, cached)
}

pub fn cache_guild_channel(
    cache: &InMemoryCache,
    channel: &Channel,
) -> Result<Option<CachedChannel>, CacheError> {
    match channel.kind {
        ChannelType::GuildText | ChannelType::GuildNews => cache_text_channel(cache, channel),
        ChannelType::GuildCategory => cache_category_channel(cache, channel),
        ChannelType::GuildNewsThread
        | ChannelType::GuildPublicThread
        | ChannelType::GuildPrivateThread => cache_thread(cache, channel),
        _ => Ok(None),
    }
}

pub fn cache_text_channel(
    cache: &InMemoryCache,
    channel: &Channel,
) -> Result<Option<CachedChannel>, CacheError> {
    let cached = CachedTextChannel {
        id: channel.id,
        guild_id: channel.guild_id.ok_or(CacheError::GuildId)?,
        name: channel.name.as_ref().ok_or(CacheError::Name)?.clone(),
        parent_id: channel.parent_id,
        position: channel.position.ok_or(CacheError::Position)?,
        permission_overwrites: channel
            .permission_overwrites
            .as_ref()
            .ok_or(CacheError::PermissionOverwrites)?
            .clone(),
        rate_limit_per_user: channel.rate_limit_per_user,
    };

    Ok(cache.channels.insert(channel.id, cached.into()))
}

pub fn cache_category_channel(
    cache: &InMemoryCache,
    channel: &Channel,
) -> Result<Option<CachedChannel>, CacheError> {
    let cached = CachedCategoryChannel {
        id: channel.id,
        guild_id: channel.guild_id.ok_or(CacheError::GuildId)?,
        name: channel.name.as_ref().ok_or(CacheError::Name)?.clone(),
        position: channel.position.ok_or(CacheError::Position)?,
        permission_overwrites: channel
            .permission_overwrites
            .as_ref()
            .ok_or(CacheError::PermissionOverwrites)?
            .clone(),
    };

    Ok(cache.channels.insert(channel.id, cached.into()))
}

pub fn cache_thread(
    cache: &InMemoryCache,
    thread: &Channel,
) -> Result<Option<CachedChannel>, CacheError> {
    let cached = CachedThread {
        id: thread.id,
        guild_id: thread.guild_id.ok_or(CacheError::GuildId)?,
        name: thread.name.as_ref().ok_or(CacheError::Name)?.clone(),
        private: false,
        parent_id: thread.parent_id,
        rate_limit_per_user: thread.rate_limit_per_user,
    };

    Ok(cache.channels.insert(thread.id, cached.into()))
}

/// Error occurred when caching resource.
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("missing guild id")]
    GuildId,
    #[error("missing channel name")]
    Name,
    #[error("missing channel position")]
    Position,
    #[error("missing channel permission overwrites")]
    PermissionOverwrites,
}
