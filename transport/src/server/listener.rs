use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use raidprotect_util::shutdown::{Shutdown, ShutdownSubscriber};
use remoc::rtc::ServerShared;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Semaphore,
    time,
};
use tracing::{debug, error, info, instrument, warn};

use crate::{
    cache::{Cache, CacheClient, CacheServerShared},
    server::handler::Handler,
};

/// Maximum number of connections that can be handled concurrently.
const MAX_CONNECTIONS: usize = 10;

/// Server listener.
///
/// This type wrap the TCP listener used by the server and
/// accept new connections.
#[derive(Debug)]
pub struct GatewayListener {
    /// TCP listener initialized by the `start` method.
    listener: TcpListener,
    /// Server shutdown handler
    shutdown: Shutdown,
    /// Semaphore used to limit the number of concurrent connections.
    ///
    /// Before attempting to accept a new connection, a permit is acquired
    /// from the semaphore. If the semaphore is empty, the server will wait
    /// for one.
    limit_connections: Arc<Semaphore>,
    /// Cache client exposed by the server.
    cache: CacheClient,
}

impl GatewayListener {
    /// Start the server and handle incoming connections.
    #[instrument(name = "start_listener", skip(cache, shutdown))]
    pub async fn start<C>(port: u16, cache: Arc<C>, mut shutdown: ShutdownSubscriber) -> Result<()>
    where
        C: Cache + Send + Sync + 'static,
    {
        let addr = (Ipv4Addr::LOCALHOST, port);
        let listener = TcpListener::bind(addr)
            .await
            .context("Failed to start TPC listener")?;

        info!("gateway listening on {:?}", addr);

        // Start the cache server
        let (server, cache) = CacheServerShared::new(cache, 10);
        tokio::spawn(server.serve(true));
        debug!("cache server started");

        let server = Self {
            listener,
            shutdown: Shutdown::new(),
            limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            cache,
        };

        let result = tokio::select! {
            res = server.handle_connections() => {
                // Only non-recoverable errors are received here
                if let Err(ref err) = res {
                    error!(error = %err, "failed to accept connection");
                }

                res
            }
            _ = shutdown.wait_shutdown() => {
                info!("shutting down");

                Ok(())
            }
        };

        // Send shutdown signal and wait until all connections are closed
        server.shutdown.shutdown(3).await;

        result
    }

    /// Handle incoming TCP connections
    async fn handle_connections(&self) -> Result<()> {
        debug!("accepting incoming connections");

        loop {
            // Wait for a permit to be available.
            //
            // The permit is forgotten which drop it without incrementing
            // the semaphore permits. A new permit is manually added when
            // the connection handler is dropped.
            self.limit_connections.acquire().await.unwrap().forget();
            let socket = self.accept().await?;
            let shutdown = self.shutdown.subscriber();
            let cache = self.cache.clone();

            tokio::spawn(async move {
                if let Err(err) = Handler::handle(socket, cache, shutdown).await {
                    error!(error = %err, "error handling connection")
                }
            });
        }
    }

    /// Accept a new TCP connection.
    ///
    /// Errors are handled with an exponential retry algorithm. If accepting a connection
    /// fails on the 6th retry, the error is considered as non-recoverable and is returned.
    #[instrument(skip(self))]
    async fn accept(&self) -> Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => {
                    info!(addr = %addr, "accepted new TCP connection");

                    return Ok(socket);
                }
                Err(err) => {
                    warn!(error = %err, "error while accepting connection");

                    if backoff > 64 {
                        return Err(err)
                            .context("Unrecoverable error while accepting TCP connection");
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}
