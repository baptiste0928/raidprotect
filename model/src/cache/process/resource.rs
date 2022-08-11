//! Insert Discord models into the cache.
//!
//! This module contains functions to update the cache from Discord models. The
//! various functions convert the value into the one used in the cache.

use std::{collections::HashSet, error::Error, fmt};

use redis::Pipeline;
use tracing::error;
use twilight_model::{
    channel::{Channel, ChannelType},
    guild::{Guild, Role},
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};

use crate::cache::{
    model::{
        CachedCategoryChannel, CachedChannel, CachedGuild, CachedRole, CachedTextChannel,
        CachedThread, CachedVoiceChannel, CurrentMember,
    },
    RedisModel,
};

pub fn cache_guild(
    pipe: &mut Pipeline,
    current_user: Id<ApplicationMarker>,
    guild: &Guild,
) -> Result<(), anyhow::Error> {
    // Insert channels and roles into the cache.
    let mut channels = HashSet::with_capacity(guild.channels.len());
    let mut roles = HashSet::with_capacity(guild.roles.len());

    for channel in &guild.channels {
        if CachedChannel::is_cached(channel.kind) {
            match cache_guild_channel(pipe, channel) {
                Ok(_) => {
                    channels.insert(channel.id);
                }
                Err(error) => {
                    error!(error = ?error, "failed to cache guild channel");
                }
            };
        }
    }

    for role in &guild.roles {
        cache_role(pipe, role, guild.id)?;

        roles.insert(role.id);
    }

    // Find the bot current member.
    let current_member = guild
        .members
        .iter()
        .find(|m| m.user.id == current_user.cast())
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

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

pub fn cache_role(
    pipe: &mut Pipeline,
    role: &Role,
    guild_id: Id<GuildMarker>,
) -> Result<(), anyhow::Error> {
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

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

pub fn cache_guild_channel(pipe: &mut Pipeline, channel: &Channel) -> Result<(), anyhow::Error> {
    // KEEP IN SYNC with `is_cached` in `model/src/cache/model/channel.rs`.
    match channel.kind {
        ChannelType::GuildText | ChannelType::GuildNews => cache_text_channel(pipe, channel),
        ChannelType::GuildVoice | ChannelType::GuildStageVoice => {
            cache_voice_channel(pipe, channel)
        }
        ChannelType::GuildCategory => cache_category_channel(pipe, channel),
        ChannelType::GuildNewsThread
        | ChannelType::GuildPublicThread
        | ChannelType::GuildPrivateThread => cache_thread(pipe, channel),
        // Other channels types are explicitly ignored to trigger a compiler
        // error if a new type is added.
        ChannelType::Private
        | ChannelType::Group
        | ChannelType::GuildDirectory
        | ChannelType::GuildForum  // TODO: #133
        | ChannelType::Unknown(_) => Ok(()),
    }
}

pub fn cache_text_channel(pipe: &mut Pipeline, channel: &Channel) -> Result<(), anyhow::Error> {
    let cached = CachedChannel::from(CachedTextChannel {
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
    });

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

fn cache_voice_channel(pipe: &mut Pipeline, channel: &Channel) -> Result<(), anyhow::Error> {
    let cached = CachedChannel::from(CachedVoiceChannel {
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
    });

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

pub fn cache_category_channel(pipe: &mut Pipeline, channel: &Channel) -> Result<(), anyhow::Error> {
    let cached = CachedChannel::from(CachedCategoryChannel {
        id: channel.id,
        guild_id: channel.guild_id.ok_or(CacheError::GuildId)?,
        name: channel.name.as_ref().ok_or(CacheError::Name)?.clone(),
        position: channel.position.ok_or(CacheError::Position)?,
        permission_overwrites: channel
            .permission_overwrites
            .as_ref()
            .ok_or(CacheError::PermissionOverwrites)?
            .clone(),
    });

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

pub fn cache_thread(pipe: &mut Pipeline, thread: &Channel) -> Result<(), anyhow::Error> {
    let cached = CachedChannel::from(CachedThread {
        id: thread.id,
        guild_id: thread.guild_id.ok_or(CacheError::GuildId)?,
        name: thread.name.as_ref().ok_or(CacheError::Name)?.clone(),
        private: false,
        parent_id: thread.parent_id.ok_or(CacheError::ParentId)?,
        rate_limit_per_user: thread.rate_limit_per_user,
    });

    pipe.set(cached.key(), cached.serialize_model()?);

    Ok(())
}

/// Error occurred when caching resource.
#[derive(Debug)]
pub enum CacheError {
    GuildId,
    Name,
    Position,
    PermissionOverwrites,
    ParentId,
}

impl Error for CacheError {}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::GuildId => f.write_str("missing guild id"),
            CacheError::Name => f.write_str("missing channel name"),
            CacheError::Position => f.write_str("missing channel position"),
            CacheError::PermissionOverwrites => {
                f.write_str("missing channel permission overwrites")
            }
            CacheError::ParentId => f.write_str("missing thread parent id"),
        }
    }
}
