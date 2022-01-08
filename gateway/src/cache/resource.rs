//! Insert Discord models into the cache.
//!
//! This module contains functions to update the cache from Discord models. The
//! various functions convert the value into the one used in the cache.

use std::collections::HashSet;

use raidprotect_model::cache::{
    CachedCategoryChannel, CachedChannel, CachedGuild, CachedRole, CachedTextChannel, CachedThread,
    CurrentMember,
};
use twilight_model::{
    channel::{
        thread::{NewsThread, PrivateThread, PublicThread},
        CategoryChannel, GuildChannel, TextChannel,
    },
    guild::{Guild, Role},
    id::GuildId,
};

use super::InMemoryCache;

pub fn cache_guild(cache: &InMemoryCache, guild: &Guild) -> Option<CachedGuild> {
    // Insert channels and roles into the cache.
    let mut channels = HashSet::with_capacity(guild.channels.len());
    let mut roles = HashSet::with_capacity(guild.roles.len());

    for channel in &guild.channels {
        if CachedChannel::is_cached(channel.kind()) {
            cache_guild_channel(cache, channel, guild.id);

            channels.insert(channel.id());
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
        name: guild.name.clone(),
        icon: guild.icon.clone(),
        owner_id: guild.owner_id,
        current_member,
        roles,
        channels,
    };

    cache.guilds.insert(guild.id, cached)
}

pub fn cache_role(cache: &InMemoryCache, role: &Role, guild_id: GuildId) -> Option<CachedRole> {
    let cached = CachedRole {
        id: role.id,
        guild_id,
        name: role.name.clone(),
        color: role.color,
        icon: role.icon.clone(),
        unicode_emoji: role.unicode_emoji.clone(),
        position: role.position,
        permissions: role.permissions,
        managed: role.managed,
    };

    cache.roles.insert(role.id, cached)
}

pub fn cache_guild_channel(
    cache: &InMemoryCache,
    channel: &GuildChannel,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    match channel {
        GuildChannel::Text(channel) => cache_text_channel(cache, channel, guild_id),
        GuildChannel::Category(channel) => cache_category_channel(cache, channel, guild_id),
        GuildChannel::PublicThread(thread) => cache_public_thread(cache, thread, guild_id),
        GuildChannel::PrivateThread(thread) => cache_private_thread(cache, thread, guild_id),
        GuildChannel::NewsThread(thread) => cache_news_thread(cache, thread, guild_id),
        _ => None,
    }
}

pub fn cache_text_channel(
    cache: &InMemoryCache,
    channel: &TextChannel,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    let cached = CachedTextChannel {
        id: channel.id,
        guild_id,
        name: channel.name.clone(),
        parent_id: channel.parent_id,
        position: channel.position,
        permission_overwrites: channel.permission_overwrites.clone(),
        rate_limit_per_user: channel.rate_limit_per_user,
    };

    cache.channels.insert(channel.id, cached.into())
}

pub fn cache_category_channel(
    cache: &InMemoryCache,
    channel: &CategoryChannel,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    let cached = CachedCategoryChannel {
        id: channel.id,
        guild_id,
        name: channel.name.clone(),
        position: channel.position,
        permission_overwrites: channel.permission_overwrites.clone(),
    };

    cache.channels.insert(channel.id, cached.into())
}

pub fn cache_public_thread(
    cache: &InMemoryCache,
    thread: &PublicThread,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    let cached = CachedThread {
        id: thread.id,
        guild_id,
        name: thread.name.clone(),
        private: false,
        parent_id: thread.parent_id,
        rate_limit_per_user: thread.rate_limit_per_user,
    };

    cache.channels.insert(thread.id, cached.into())
}

pub fn cache_private_thread(
    cache: &InMemoryCache,
    thread: &PrivateThread,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    let cached = CachedThread {
        id: thread.id,
        guild_id,
        name: thread.name.clone(),
        private: true,
        parent_id: thread.parent_id,
        rate_limit_per_user: thread.rate_limit_per_user,
    };

    cache.channels.insert(thread.id, cached.into())
}

pub fn cache_news_thread(
    cache: &InMemoryCache,
    thread: &NewsThread,
    guild_id: GuildId,
) -> Option<CachedChannel> {
    let cached = CachedThread {
        id: thread.id,
        guild_id,
        name: thread.name.clone(),
        private: false,
        parent_id: thread.parent_id,
        rate_limit_per_user: thread.rate_limit_per_user,
    };

    cache.channels.insert(thread.id, cached.into())
}
