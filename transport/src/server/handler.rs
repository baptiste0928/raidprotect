use anyhow::{Context, Result};
use raidprotect_util::shutdown::{Shutdown, ShutdownSubscriber};
use remoc::rch;
use tokio::net::TcpStream;
use tracing::{debug, error, instrument, Instrument};

use crate::model::BaseRequest;

/// Connection handler.
///
/// This type handle a single TCP connection and manage
/// the associated [`remoc`] base channel.
pub struct Handler {
    /// Base channel receiver
    receiver: rch::base::Receiver<BaseRequest>,
    /// Base channel sender (unused)
    _sender: rch::base::Sender<()>,
    /// Connection shutdown handler
    shutdown: Shutdown,
}

impl Handler {
    /// Create a new [`Handler`] from a TCP stream.
    #[instrument(skip_all)]
    pub async fn handle(socket: TcpStream, mut shutdown: ShutdownSubscriber) -> Result<()> {
        let (socket_rx, socket_tx) = socket.into_split();

        // Establish remoc connection
        let cfg = remoc::Cfg::default();
        let (conn, _sender, receiver) = remoc::Connect::io(cfg, socket_rx, socket_tx)
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
        let mut handler = Self {
            receiver,
            _sender,
            shutdown: Shutdown::new(),
        };

        let result = tokio::select! {
            res = handler.handle_requests() => res,
            _ = shutdown.wait_shutdown() => {
                debug!("connection shutting down");

                Ok(())
            },
        };

        // Send shutdown signal and stop connection
        handler.receiver.close().await;
        handler.shutdown.shutdown(2).await;
        conn.abort();

        result
    }

    async fn handle_requests(&mut self) -> Result<()> {
        while let Some(req) = self.receiver.recv().await? {
            match req {
                BaseRequest::EventBroadcast { .. } => (),
            }
        }

        Ok(())
    }
}
