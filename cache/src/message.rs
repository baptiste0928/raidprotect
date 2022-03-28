//! Message cache.
//!
//! This module expose the cache used to store messages for anti-spam processing.
//! Unlike [`InMemoryCache`], each cached message has a TTL and expires after 2
//! minutes.
//!
//! Internally, the cache contains two storages:
//! - `ttl_index` is a [`VecDeque`] that track the expiration of stored
//! messages.
//! - `messages` is a [`DashMap`] that store messages based on the channels.
//! Inside, messages are stored inside a [`VecDeque`] to allow fast insertion.
//!
//! To share the cache between multiple threads, wrap it into an [`Arc`].
//!
//! [`InMemoryCache`]: crate::InMemoryCache
//! [`Arc`]: std::sync::Arc

use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::{mapref::one::Ref, DashMap};
use tokio::{sync::RwLock, time};
use tracing::{debug, trace};
use twilight_model::id::{marker::ChannelMarker, Id};

use crate::model::CachedMessage;

/// Message cache.
///
/// See the [module documentation](super) for more information about internal
/// implementation.
#[derive(Debug, Default)]
pub struct MessageCache {
    ttl_index: RwLock<VecDeque<MessageTtl>>,
    messages: DashMap<Id<ChannelMarker>, VecDeque<CachedMessage>>,
}

impl MessageCache {
    /// Initialize a new empty [`MessageCache`].
    ///
    /// This returns a new cache wrapped in an [`Arc`], and a [`MessageExpireTask`]
    /// used to expire old messages.
    #[must_use]
    pub fn new() -> (Arc<Self>, MessageExpireTask) {
        let cache = Arc::new(Self::default());

        (cache.clone(), MessageExpireTask::new(cache))
    }

    /// Insert a new [`CachedMessage`] into the cache.
    pub async fn insert(&self, message: CachedMessage) {
        let channel_id = message.channel_id;

        // Insert message into the cache.
        self.messages
            .entry(channel_id)
            .and_modify(|cache| cache.push_back(message.clone()))
            .or_insert_with(|| {
                let mut cache = VecDeque::with_capacity(1);
                cache.push_back(message);

                cache
            });

        // If *may* be possible that the ttl_index is not in the exact same
        // order as messages insertions. Anyway, it should occur only of two
        // messages are inserted at the same time and should not be a problem.
        let now = Instant::now();
        let mut ttl_index = self.ttl_index.write().await;

        ttl_index.push_back(MessageTtl::new(channel_id, now));
    }

    /// Get all cached messages of a channel.
    ///
    /// The returned [`Ref`] should be kept as little as possible, because holding
    /// it lock the cache for updating other messages.
    pub fn get_channel_messages(
        &self,
        channel_id: Id<ChannelMarker>,
    ) -> Option<Ref<'_, Id<ChannelMarker>, VecDeque<CachedMessage>>> {
        self.messages.get(&channel_id)
    }
}

/// Message expiration task.
///
/// This type is created when initializing a new [`MessageCache`] and handle
/// deletion of expired cache entries. The [`run`] method must be called in a
/// new task to start it.
///
/// [`run`]: Self::run
#[derive(Debug, Clone)]
pub struct MessageExpireTask {
    cache: Arc<MessageCache>,
}

impl MessageExpireTask {
    /// Initialize a new [`MessageExpireTask`].
    fn new(cache: Arc<MessageCache>) -> Self {
        Self { cache }
    }

    /// Start the message expiration task.
    pub async fn run(&self) {
        debug!("started message expiration task");

        loop {
            // Get the next expiration time
            let duration = {
                // Explicit scope to ensure the lock is dropped
                let ttl_index = self.cache.ttl_index.read().await;

                match ttl_index.get(0) {
                    Some(message) => message.expires_in() + MessageTtl::DELTA_DELAY,
                    None => MessageTtl::DEFAULT_DELAY,
                }
            };

            time::sleep(duration).await;

            {
                // Explicit scope to ensure the lock is dropped
                let mut ttl_index = self.cache.ttl_index.write().await;

                // Expire all messages
                loop {
                    if !ttl_index
                        .get(0)
                        .map(|msg| msg.expires_in().is_zero())
                        .unwrap_or(false)
                    {
                        break; // Break if the next message has not expired
                    }

                    // Remove the message from the cache
                    let message = ttl_index.pop_front().unwrap(); // SAFETY: if the message does not exist, the loop is stopped in the before statement
                    if let Some(mut cache) = self.cache.messages.get_mut(&message.channel_id) {
                        if let Some(expired) = cache.pop_front() {
                            trace!(message_id = %expired.id, "message expired from cache");
                        }
                    }
                }
            }
        }
    }
}

/// Expiration of a message.
///
/// This type only hold the channel [`Id`] of the message, and the [`Instant`]
/// of its insertion to the cache. This assume that **cached messages are not
/// reordered**, or else a wrong message could be cleared from the cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MessageTtl {
    channel_id: Id<ChannelMarker>,
    created_at: Instant,
}

impl MessageTtl {
    /// Message expiration duration (2 minutes)
    const EXPIRES_AFTER: Duration = Duration::from_secs(2 * 60);

    /// Default delay between two checks if the queue is empty.
    const DEFAULT_DELAY: Duration = Duration::from_millis(100);

    /// Delay added to each check to group together near expires.
    const DELTA_DELAY: Duration = Duration::from_millis(5);

    /// Initialize a new [`MessageTtl`].
    fn new(channel_id: Id<ChannelMarker>, created_at: Instant) -> Self {
        Self {
            channel_id,
            created_at,
        }
    }

    /// Returns when the message expires.
    ///
    /// If the message has already expired, a zero duration is returned.
    fn expires_in(&self) -> Duration {
        let elapsed = self.created_at.elapsed();

        // Computes EXPIRES_AFTER - elapsed, returning zero if resulting in
        // a negative duration (already expired)
        Self::EXPIRES_AFTER.saturating_sub(elapsed)
    }
}
