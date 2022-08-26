//! Insert Discord models into the cache.
//!
//! This module contains functions to update the cache from Discord models. The
//! various functions convert the value into the one used in the cache.

use std::collections::HashSet;

use anyhow::Context;
use redis::Pipeline;
use tracing::error;
use twilight_model::{
    channel::Channel,
    guild::{Guild, Role},
    id::{
        marker::{ApplicationMarker, GuildMarker},
        Id,
    },
};

use crate::cache::{
    model::{CachedChannel, CachedGuild, CachedRole, CurrentMember},
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
    if CachedChannel::is_cached(channel.kind) {
        let cached = CachedChannel {
            id: channel.id,
            guild_id: channel.guild_id.context("missing guild id")?,
            kind: channel.kind,
            name: channel.name.clone().context("missing channel name")?,
            parent_id: channel.parent_id,
            permission_overwrites: channel.permission_overwrites.clone(),
            position: channel.position,
            rate_limit_per_user: channel.rate_limit_per_user,
        };

        pipe.set(cached.key(), cached.serialize_model()?);
    }

    Ok(())
}
