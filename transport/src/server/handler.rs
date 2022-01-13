use anyhow::{Context, Result};
use raidprotect_util::shutdown::{Shutdown, ShutdownSubscriber};
use remoc::rch;
use tokio::net::TcpStream;
use tracing::{debug, error, instrument, warn, Instrument};

use crate::{
    cache::CacheClient,
    model::{BaseRequest, CacheResponse},
};

/// Connection handler.
///
/// This type handle a single TCP connection and manage
/// the associated [`remoc`] base channel.
pub struct Handler {
    /// Base channel sender (unused)
    _sender: rch::base::Sender<()>,
    /// Connection shutdown handler
    shutdown: Shutdown,
    /// Cache exposed by the server.
    cache: CacheClient,
}

impl Handler {
    /// Create a new [`Handler`] from a TCP stream.
    #[instrument(skip_all)]
    pub async fn handle(
        socket: TcpStream,
        cache: CacheClient,
        mut shutdown: ShutdownSubscriber,
    ) -> Result<()> {
        let (socket_rx, socket_tx) = socket.into_split();

        // Establish remoc connection
        let cfg = remoc::Cfg::default();
        let (conn, _sender, mut receiver) = remoc::Connect::io(cfg, socket_rx, socket_tx)
            .await
            .context("failed to initialize remoc connection")?;

        // Start remoc connection
        let conn = tokio::spawn(async move {
            let res = conn
                .instrument(tracing::info_span!("remoc connection").or_current())
                .await;

            if let Err(err) = res {
                error!(error = %err, "remoc connection error")
            }
        });

        // Run connection handler
        let handler = Self {
            _sender,
            shutdown: Shutdown::new(),
            cache,
        };

        let result = tokio::select! {
            res = handler.handle_requests(&mut receiver) => res,
            _ = shutdown.wait_shutdown() => {
                debug!("connection shutting down");

                Ok(())
            },
        };

        // Send shutdown signal and stop connection
        receiver.close().await;
        handler.shutdown.shutdown(2).await;
        conn.abort();

        result
    }

    async fn handle_requests(&self, receiver: &mut rch::base::Receiver<BaseRequest>) -> Result<()> {
        while let Some(req) = receiver.recv().await? {
            match req {
                BaseRequest::EventBroadcast { .. } => (),
                BaseRequest::Cache { callback } => self.handle_cache_request(callback).await,
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    async fn handle_cache_request(&self, callback: rch::oneshot::Sender<CacheResponse>) {
        let res = CacheResponse {
            client: self.cache.clone(),
        };

        if let Err(err) = callback.send(res) {
            warn!(error = %err, "failed to send cache response")
        };
    }
}
