//! Event context.
//!
//! These structs represent the context of an event, with additional information
//! retrived from cache or other

use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
};

use raidprotect_cache::{model::CachedGuild, InMemoryCache};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::GuildMarker, Id};

/// Context for guild events.
#[derive(Debug, Clone)]
pub struct GuildContext {
    /// The cached guild.
    pub guild: CachedGuild,
    /// Bot in-memory cache.
    pub cache: Arc<InMemoryCache>,
    /// Shared Discord HTTP client.
    pub http: Arc<HttpClient>,
}

impl GuildContext {
    /// Initialize a new [`GuildContext`].
    pub(crate) async fn new(
        guild_id: Id<GuildMarker>,
        cache: Arc<InMemoryCache>,
        http: Arc<HttpClient>,
    ) -> Result<Self, ContextError> {
        let guild = match cache.guild(guild_id) {
            Some(guild) => guild.clone(),
            None => return Err(ContextError::CacheNotFound),
        };

        Ok(Self { guild, cache, http })
    }
}

/// Error when intializing a [`GuildContext`].
#[derive(Debug)]
pub enum ContextError {
    /// Guild not found in the cache.
    CacheNotFound,
}

impl Error for ContextError {}

impl Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextError::CacheNotFound => f.write_str("guild not found in cache"),
        }
    }
}
