//! Event context.
//!
//! These structs represent the context of an event, with additional information
//! retrieved from cache or database.

use std::{
    error::Error,
    fmt::{self, Display},
    sync::Arc,
};

use raidprotect_cache::{model::CachedGuild, InMemoryCache};
use twilight_http::Client as HttpClient;
use twilight_model::id::{marker::GuildMarker, Id};

/// Generic context for bot events.
#[derive(Debug, Clone)]
pub struct EventContext {
    /// Bot in-memory cache.
    pub cache: Arc<InMemoryCache>,
    /// Shared Discord HTTP client.
    pub http: Arc<HttpClient>,
}

impl EventContext {
    /// Initialize a new [`EventContext`].
    pub(crate) fn new(cache: Arc<InMemoryCache>, http: Arc<HttpClient>) -> Self {
        Self { cache, http }
    }
}

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

/// Error when initializing a [`GuildContext`].
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
