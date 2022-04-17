//! Message cache.
//!
//! This module expose the cache used to store messages for anti-spam processing.
//! Unlike [`InMemoryCache`], each cached message has a TTL and expires after 2
//! minutes.
//!
//! The cache uses the [`async_ttl`] crate.

use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use async_ttl::{config::AsyncTtlConfig, AsyncTtl, AsyncTtlExpireTask, CacheMap};
use tokio::sync::RwLockReadGuard;
use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker},
    Id,
};

use crate::model::CachedMessage;

/// Message cache expiration task.
pub type MessageExpireTask = AsyncTtlExpireTask<MessageCacheInner, CacheKey, CachedMessage>;

/// Message cache.
///
/// See the [module documentation](super) for more information.
#[derive(Debug, Clone)]
pub struct MessageCache {
    cache: Arc<AsyncTtl<MessageCacheInner, CacheKey, CachedMessage>>,
}

impl MessageCache {
    /// Message expiration duration (2 minutes)
    const EXPIRES_AFTER: Duration = Duration::from_secs(2 * 60);

    /// Initialize a new [`MessageCache`].
    ///
    /// This returns a new cache and a [`AsyncTtlExpireTask`] used to expire old
    /// messages.
    pub fn new() -> (
        Self,
        AsyncTtlExpireTask<MessageCacheInner, CacheKey, CachedMessage>,
    ) {
        let (cache, expire_task) = AsyncTtl::new(AsyncTtlConfig::new(Self::EXPIRES_AFTER));

        (Self { cache }, expire_task)
    }

    /// Insert a new [`CachedMessage`] into the cache.
    pub async fn insert(&self, message: CachedMessage) {
        let key = (message.channel_id, message.id);

        self.cache.insert(key, message).await;
    }

    /// Get a read lock to the internal cache.
    pub async fn read(&self) -> RwLockReadGuard<'_, MessageCacheInner> {
        self.cache.read().await
    }
}

/// Message cache internal data structure.
#[derive(Debug, Clone, Default)]
pub struct MessageCacheInner {
    messages: HashMap<Id<ChannelMarker>, VecDeque<CachedMessage>>,
}

type CacheKey = (Id<ChannelMarker>, Id<MessageMarker>);

impl CacheMap<CacheKey, CachedMessage> for MessageCacheInner {
    fn insert_cache(&mut self, key: CacheKey, value: CachedMessage) {
        let (channel_id, _) = key;

        self.messages
            .entry(channel_id)
            .and_modify(|cache| cache.push_back(value.clone()))
            .or_insert_with(|| {
                let mut cache = VecDeque::with_capacity(1);
                cache.push_back(value);

                cache
            });
    }

    fn remove_cache(&mut self, key: &CacheKey) {
        let (channel_id, message_id) = key;

        if let Some(cache) = self.messages.get_mut(channel_id) {
            if let Some((index, _)) = cache
                .iter()
                .enumerate()
                .find(|(_, message)| &message.id == message_id)
            {
                cache.remove(index);
            }
        }
    }
}
