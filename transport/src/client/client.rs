use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::{
    sync::{broadcast, Mutex, RwLock, RwLockReadGuard},
    time::sleep,
};

use super::{connect::Connection, ClientError};

/// Gateway client.
///
/// The internal client state is held in an [`Arc`], allowing to cheaply clone
/// this type.
pub struct GatewayClient {
    inner: Arc<ClientInner>,
}

/// Internal client state.
///
/// This type hold the shared state of the client. The type handle reconnection
/// logic if the connection is dropped for some reason.
///
/// # Automatic reconnection
/// This type is notified when the underlying [`Connection`] is stopped with a
/// mspc channel. When disconnected, all requests to the client will be marked as
/// pending, with a timeout of one second.
pub struct ClientInner {
    connection: RwLock<Connection>,
}

/// Reconnection handler.
///
/// This type wraps a client of type `T` with a connection state. It implement
/// [`Deref`] to access to the inner client.
struct Reconnect<T> {
    /// Broadcast channel to notify on connection
    broadcast: broadcast::Sender<()>,
    /// Current connection state
    connected: RwLock<bool>,
    /// Inner client
    inner: RwLock<T>,
}

impl<T> Reconnect<T> {
    /// Initialize an new [`Reconnect`]
    fn new(inner: T) -> Self {
        let (sender, _receiver) = broadcast::channel(1);

        Self {
            broadcast: sender,
            connected: RwLock::new(true),
            inner: RwLock::new(inner),
        }
    }

    /// Set the connection state of the client.
    async fn set_connected(&self, connected: bool) {
        let mut state = self.connected.write().await;
        *state = connected;

        if connected {
            let _ = self.broadcast.send(());
        }
    }

    /// Wait until the client is connected.
    ///
    /// An error is returned if the reconnection time is longer than `timeout`.
    async fn wait_connected(&self, timeout: u64) -> Result<(), ClientError> {
        if *self.connected.read().await {
            return Ok(());
        }

        let mut receiver = self.broadcast.subscribe();
        tokio::select! {
            _ = receiver.recv() => Ok(()),
            _ = sleep(Duration::from_secs(timeout)) => Err(ClientError::ReconnectTimeout)
        }
    }
}

impl<T> Deref for Reconnect<T> {
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
