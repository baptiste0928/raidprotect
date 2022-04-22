//! State for message component interactions (buttons, select menus).

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_ttl::{config::AsyncTtlConfig, AsyncTtl, AsyncTtlExpireTask};
use twilight_model::id::{marker::UserMarker, Id};

use super::InteractionResponse;

/// Queue of pending components.
///
/// This type holds pending components interaction associated with their id.
/// Components are automatically expired after 5 minutes.
///
/// This type can be cloned as internal state is wrapped in an [`Arc`].
#[derive(Debug, Clone)]
pub struct PendingComponentQueue {
    inner: Arc<AsyncTtl<HashMap<String, PendingComponent>, String, PendingComponent>>,
}

pub type PendingComponentExpireTask =
    AsyncTtlExpireTask<HashMap<String, PendingComponent>, String, PendingComponent>;

impl PendingComponentQueue {
    const EXPIRES_AFTER: Duration = Duration::from_secs(5 * 60);

    /// Initialize a new [`PendingComponentQueue`].
    ///
    /// This function also returns an [`PendingComponentExpireTask`].
    pub fn new() -> (Self, PendingComponentExpireTask) {
        let (inner, expire_task) = AsyncTtl::new(AsyncTtlConfig::new(Self::EXPIRES_AFTER));

        (Self { inner }, expire_task)
    }

    /// Insert a new component into the queue.
    pub async fn insert(&self, custom_id: String, component: PendingComponent) {
        self.inner.insert(custom_id, component).await;
    }

    /// Get a component from the queue.
    pub async fn get(&self, custom_id: &str) -> Option<PendingComponent> {
        self.inner.read().await.get(custom_id).cloned()
    }
}

/// State of a component waiting for user interaction.
#[derive(Debug, Clone)]
pub enum PendingComponent {
    PostInChatButton(PostInChatButton),
}

/// State for the "post in chat" button.
///
/// See the `raidprotect-handler` for more information.
#[derive(Debug, Clone)]
pub struct PostInChatButton {
    /// Response to send to the channel.
    pub response: InteractionResponse,
    /// Id of the initial interaction author.
    pub author_id: Id<UserMarker>,
}
