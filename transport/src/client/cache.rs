use std::sync::Arc;

use tokio::{
    sync::{RwLock, RwLockReadGuard, Semaphore},
    time::sleep,
};

use crate::cache::CacheClient as RemocCacheClient;

use super::{error::ReconnectTimeoutError, gateway::ClientInner};

/// Cache client with automatic reconnection.
///
/// This client wraps [`cache::CacheClient`] with automatic reconnection to the
/// server.
///
/// [`cache::CacheClient`]: crate::cache::CacheClient
#[derive(Debug, Clone)]
pub struct CacheClient {
    /// Shared internal client state.
    client: Arc<ClientInner>,
    /// Shared internal cache client.
    ///
    /// The client is wrapped in a [`RwLock`] to allow updating it in case
    /// of disconnection.
    cache: Arc<RwLock<RemocCacheClient>>,
    /// Reconnection permits.
    reconnect: Arc<Semaphore>,
}

impl CacheClient {
    /// Initialize a new [`CacheClient`].
    pub(super) fn new(client: Arc<ClientInner>, cache: RemocCacheClient) -> Self {
        Self {
            client,
            cache: Arc::new(RwLock::new(cache)),
            reconnect: Arc::new(Semaphore::new(1)),
        }
    }

    /// Get the inner cache client.
    ///
    /// This method ensure that the client is connected before returning it.
    async fn cache(&self) -> Result<RwLockReadGuard<'_, RemocCacheClient>, ReconnectTimeoutError> {
        if self.client.is_connected().await == false {
            // Try to reconnect the client
            // If a permit is immediately acquired, this task will reconnect the client
            let permit = self.reconnect.try_acquire();

            if permit.is_ok() {
                // Reconnect the client
                self.client.wait_connected().await?;
                todo!("ask a new client")
            } else {
                // Wait until the client is reconnected (new permit can be acquired)
                return tokio::select! {
                    Ok(_) = self.reconnect.acquire() => Ok(self.cache.read().await),
                    _ =  sleep(ClientInner::RECONNECT_TIMEOUT) => Err(ReconnectTimeoutError),
                };
            }
        }

        Ok(self.cache.read().await)
    }
}
