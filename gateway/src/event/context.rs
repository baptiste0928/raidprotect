//! Event context.
//!
//! These structs represent the context of an event, with additional information
//! retrieved from cache or database.

use std::sync::Arc;

use raidprotect_cache::{model::CachedGuild, InMemoryCache};
use twilight_http::{client::InteractionClient, Client as HttpClient};
use twilight_model::id::{marker::ApplicationMarker, Id};

/// Context of a received event.
///
/// If the event occurred in a guild, a [`CachedGuild`] will be included.
#[derive(Debug, Clone)]
pub struct EventContext {
    /// Bot in-memory cache.
    pub cache: Arc<InMemoryCache>,
    /// Shared Discord HTTP client.
    pub http: Arc<HttpClient>,
    /// Bot application id
    pub application_id: Id<ApplicationMarker>,
    /// If event occurred in a guild, the cached guild.
    pub guild: Option<CachedGuild>,
}

impl EventContext {
    /// Get the [`InteractionClient`] associated with the current context.
    pub fn interaction(&self) -> InteractionClient {
        self.http.interaction(self.application_id)
    }
}
