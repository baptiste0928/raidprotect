//! Message cache.
//!
//! This module expose the cache used to store messages for anti-spam processing.
//! Unlike [`InMemoryCache`], each cached message has a TTL and expires after 5
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
    time::{Duration, Instant},
};

use dashmap::DashMap;
use twilight_model::id::{marker::ChannelMarker, Id};

use crate::model::CachedMessage;

/// Message cache.
///
/// See the [module documentation](super) for more information about internal
/// implementation.
#[derive(Debug)]
pub struct MessageCache {
    ttl_index: VecDeque<MessageTtl>,
    messages: DashMap<Id<ChannelMarker>, VecDeque<CachedMessage>>,
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
    /// Message expiration duration (5 minutes)
    const EXPIRES_AFTER: Duration = Duration::from_secs(5 * 60);

    /// Returns when the message expires.
    ///
    /// If the message has already expired, [`None`] is returned instead.
    fn expires_in(&self) -> Option<Duration> {
        let elapsed = self.created_at.elapsed();

        // Computes EXPIRES_AFTER - elapsed, returning `None` if resulting in
        // a negative duration (already expired)
        Self::EXPIRES_AFTER.checked_sub(elapsed)
    }

    /// Get the message channel id.
    fn channel_id(&self) -> Id<ChannelMarker> {
        self.channel_id
    }
}
