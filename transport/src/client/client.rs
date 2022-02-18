use std::{fmt::Debug, ops::Deref, sync::Arc, time::Duration};

use tokio::{
    net::ToSocketAddrs,
    sync::{broadcast, Mutex, RwLock},
    time::sleep,
};
use tracing::{info, instrument, warn};

use crate::{cache::CacheClient, model::BaseRequest};

use super::{connect::Connection, error::ReconnectTimeoutError, ClientError};

/// Gateway client.
///
/// The internal client state is held in an [`Arc`], allowing to cheaply clone
/// this type.
#[derive(Debug)]
pub struct GatewayClient<A>
where
    A: ToSocketAddrs + Send + Sync + Clone,
{
    inner: Arc<ClientInner<A>>,
}

impl<A> GatewayClient<A>
where
    A: ToSocketAddrs + Send + Sync + Clone,
{
    /// Start a new [`GatewayClient`]
    pub async fn start(addr: A) -> Result<Self, ClientError> {
        let inner = ClientInner::start(addr).await?;

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Get the current [`CacheClient`]
    pub async fn cache(&self) -> Result<CacheClient, ClientError> {
        todo!()
    }

    /// Automatically reconnect the client.
    ///
    /// This task must be launched after the client is connected to ensure it
    /// will properly reconnect in case of failure.
    ///
    /// Reconnections are handled with an exponential retry algorithm. If
    /// reconnecting fails on the 6th retry (64 seconds), the error is
    /// considered as non-recoverable and is returned.
    #[must_use = "this method must be called to automatically reconnect client"]
    pub async fn reconnect(&self) -> Result<(), ClientError> {
        self.inner.reconnect().await
    }
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
#[derive(Debug)]
pub struct ClientInner<A>
where
    A: ToSocketAddrs + Send + Sync + Clone,
{
    /// Inner remoc connection.
    connection: Mutex<Connection>,
    /// Current connection state.
    connected: RwLock<bool>,
    /// Connection update broadcast channel.
    broadcast: broadcast::Sender<ConnectionUpdate>,
    /// Connection socked adress.
    addr: A,
}

impl<A> ClientInner<A>
where
    A: ToSocketAddrs + Send + Sync + Clone,
{
    const RECONNECT_TIMEOUT: Duration = Duration::from_secs(1);
    const RECONNECT_MAX_BACKOFF: u64 = 64; // 6 retries with exponential backoff

    /// Initialize a new [`ClientInner`] with an active connection
    async fn start(addr: A) -> Result<Self, ClientError> {
        let (sender, _) = broadcast::channel(1);
        let connection = Connection::start(addr.clone(), sender.clone()).await?;

        Ok(Self {
            connection: Mutex::new(connection),
            connected: RwLock::new(true),
            broadcast: sender,
            addr,
        })
    }

    /// Send a request through the connection.
    pub async fn send(&mut self, req: BaseRequest) -> Result<(), ClientError> {
        self.wait_connected().await?;

        self.connection.lock().await.send(req).await
    }

    /// Return whether the client is currently connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Wait until the client is connected.
    ///
    /// An error is returned if the reconnection time is longer than one second.
    pub async fn wait_connected(&self) -> Result<(), ReconnectTimeoutError> {
        if self.is_connected().await {
            return Ok(());
        }

        let mut receiver = self.broadcast.subscribe();
        tokio::select! {
            Ok(ConnectionUpdate::Connected) = receiver.recv() => Ok(()),
            _ = sleep(Self::RECONNECT_TIMEOUT) => Err(ReconnectTimeoutError),
        }
    }

    /// Automatically reconnect the client.
    ///
    /// See [`GatewayClient::reconnect`] for more information.
    #[instrument(skip(self))]
    pub async fn reconnect(&self) -> Result<(), ClientError> {
        let mut receiver = self.broadcast.subscribe();

        while let Ok(msg) = receiver.recv().await {
            match msg {
                ConnectionUpdate::Connected => *self.connected.write().await = true,
                ConnectionUpdate::Disconnected => {
                    *self.connected.write().await = false;
                    self.try_reconnect().await?;
                }
            }
        }

        Ok(())
    }

    /// Try to reconnect using a exponential retry algorithm.
    ///
    /// See [`GatewayClient::reconnect`]] for more information.
    #[instrument(skip(self))]
    async fn try_reconnect(&self) -> Result<(), ClientError> {
        let mut backoff = 1;

        loop {
            match Connection::start(self.addr.clone(), self.broadcast.clone()).await {
                Ok(conn) => {
                    info!("reconnected to the client");
                    *self.connection.lock().await = conn;
                    *self.connected.write().await = true;
                }
                Err(err) => {
                    warn!(error = %err, "error while reconnecting");

                    if backoff > Self::RECONNECT_MAX_BACKOFF {
                        return Err(err);
                    }
                }
            }

            sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

/// Message sent by the inner connection to notify any connection state update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionUpdate {
    Connected,
    Disconnected,
}
