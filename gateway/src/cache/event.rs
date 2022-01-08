//! Update the cache based on incoming event data.

use raidprotect_model::cache::CachedGuild;
use twilight_model::gateway::payload::incoming::GuildCreate;

use super::InMemoryCache;

/// Update the cache based on event data.
pub trait UpdateCache {
    /// Type of the cached value.
    type Output;

    /// Update the cache based on event data.
    ///
    /// If an old value of the updated entry is present in the cache, it will be
    /// returned.
    fn update(&self, cache: &InMemoryCache) -> Option<Self::Output>;
}

impl UpdateCache for GuildCreate {
    type Output = CachedGuild;

    fn update(&self, cache: &InMemoryCache) -> Option<Self::Output> {
        super::resource::cache_guild(cache, &self.0)
    }
}
