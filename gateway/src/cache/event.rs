//! Update the cache based on incoming event data.

use raidprotect_model::cache::{CachedChannel, CachedGuild, CurrentMember};
use twilight_model::{
    channel::Channel,
    gateway::payload::incoming::{
        ChannelCreate, ChannelDelete, ChannelUpdate, GuildCreate, GuildDelete, GuildUpdate,
        MemberAdd,
    },
};

use super::InMemoryCache;

/// Update the cache based on event data.
pub trait UpdateCache {
    /// Type of the cached value.
    type Output;

    /// Update the cache based on event data.
    ///
    /// If an old value of the updated entry is present in the cache, it will be
    /// returned.
    fn update(&self, cache: &InMemoryCache) -> Self::Output;
}

impl UpdateCache for GuildCreate {
    type Output = ();

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        super::resource::cache_guild(cache, &self.0);
    }
}

impl UpdateCache for GuildDelete {
    type Output = Option<CachedGuild>;

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        let guild = match cache.guilds.remove(&self.id) {
            Some((_, guild)) => guild,
            None => return None,
        };

        // Remove all channels and roles from the cache.
        for channel in &guild.channels {
            cache.channels.remove(channel);
        }

        for role in &guild.roles {
            cache.roles.remove(role);
        }

        Some(guild)
    }
}

impl UpdateCache for GuildUpdate {
    type Output = Option<CachedGuild>;

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        let mut guild = match cache.guilds.get_mut(&self.id) {
            Some(guild) => guild,
            None => return None,
        };

        let before = guild.clone();

        guild.name = self.name.clone();
        guild.icon = self.icon.clone();
        guild.owner_id = self.owner_id;

        Some(before)
    }
}

impl UpdateCache for ChannelCreate {
    type Output = ();

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        let channel = match &self.0 {
            Channel::Guild(channel) => channel,
            _ => return,
        };

        let guild_id = match channel.guild_id() {
            Some(guild_id) => guild_id,
            None => return,
        };

        // Cache the channel.
        super::resource::cache_guild_channel(cache, channel, guild_id);

        // Add the channel to the guild.
        if let Some(mut guild) = cache.guilds.get_mut(&guild_id) {
            guild.channels.insert(channel.id());
        }
    }
}

impl UpdateCache for ChannelDelete {
    type Output = Option<CachedChannel>;

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        let channel = match &self.0 {
            Channel::Guild(channel) => channel,
            _ => return None,
        };

        // Remove the channel from the guild.
        if let Some(mut guild) = cache.guilds.get_mut(&channel.guild_id()?) {
            guild.channels.remove(&channel.id());
        }

        // Remove the channel from the cache.
        cache
            .channels
            .remove(&channel.id())
            .map(|(_, channel)| channel)
    }
}

impl UpdateCache for ChannelUpdate {
    type Output = Option<CachedChannel>;

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        let channel = match &self.0 {
            Channel::Guild(channel) => channel,
            _ => return None,
        };

        super::resource::cache_guild_channel(cache, channel, channel.guild_id()?)
    }
}

impl UpdateCache for MemberAdd {
    type Output = Option<CurrentMember>;

    fn update(&self, cache: &InMemoryCache) -> Self::Output {
        if self.user.id != cache.current_user {
            return None;
        }

        let mut guild = cache.guilds.get_mut(&self.guild_id)?;
        let previous = guild.current_member.clone();

        let cached = CurrentMember {
            id: self.user.id,
            communication_disabled_until: self.communication_disabled_until,
            roles: guild.roles.iter().copied().collect(),
        };

        guild.current_member = Some(cached);

        previous
    }
}
