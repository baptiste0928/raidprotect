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
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::id::{
    marker::{ApplicationMarker, GuildMarker},
    Id,
};

/// Trait implemented by event context types.
pub trait EventContext {
    /// Get the [`InteractionClient`] associated with the current context.
    fn interaction(&self) -> InteractionClient;
}

/// Generic context for bot events.
#[derive(Debug, Clone)]
pub struct BaseContext {
    /// Bot in-memory cache.
    pub cache: Arc<InMemoryCache>,
    /// Shared Discord HTTP client.
    pub http: Arc<HttpClient>,
    /// Bot application id
    pub application_id: Id<ApplicationMarker>,
}

impl BaseContext {
    /// Initialize a new [`BaseContext`].
    pub(crate) fn new(
        cache: Arc<InMemoryCache>,
        http: Arc<HttpClient>,
        application_id: Id<ApplicationMarker>,
    ) -> Self {
        Self {
            cache,
            http,
            application_id,
        }
    }
}

impl EventContext for BaseContext {
    fn interaction(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
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
    /// Bot application id
    pub application_id: Id<ApplicationMarker>,
}

impl GuildContext {
    /// Initialize a new [`GuildContext`].
    pub(crate) fn new(
        guild_id: Id<GuildMarker>,
        cache: Arc<InMemoryCache>,
        http: Arc<HttpClient>,
        application_id: Id<ApplicationMarker>,
    ) -> Result<Self, ContextError> {
        let ctx = BaseContext::new(cache, http, application_id);

        Self::from_context(ctx, guild_id)
    }

    /// Initialize a new [`GuildContext`] from an existing [`BaseContext`].
    pub fn from_context(ctx: BaseContext, guild_id: Id<GuildMarker>) -> Result<Self, ContextError> {
        let guild = match ctx.cache.guild(guild_id) {
            Some(guild) => guild.clone(),
            None => return Err(ContextError::CacheNotFound),
        };

        Ok(Self {
            guild,
            cache: ctx.cache,
            http: ctx.http,
            application_id: ctx.application_id,
        })
    }
}

impl EventContext for GuildContext {
    fn interaction(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
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
