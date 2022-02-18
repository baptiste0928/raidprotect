use anyhow::{Context, Result};
use raidprotect_model::event::Event;
use raidprotect_transport::{
    cache::CacheClient,
    model::{BaseRequest, CacheResponse, EventBroadcastResponse},
    remoc::{self, rch},
};
use raidprotect_util::shutdown::{Shutdown, ShutdownSubscriber};
use tokio::{net::TcpStream, sync::broadcast};
use tracing::{debug, error, info_span, instrument, warn, Instrument};

use crate::server::events::EventBroadcastHandler;

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
    /// Event stream receiver
    events: broadcast::Sender<Event>,
}

impl Handler {
    /// Create a new [`Handler`] from a TCP stream.
    #[instrument(skip_all)]
    pub async fn handle(
        socket: TcpStream,
        cache: CacheClient,
        events: broadcast::Sender<Event>,
        mut shutdown: ShutdownSubscriber,
    ) -> Result<()> {
        let (socket_rx, socket_tx) = socket.into_split();

        // Establish remoc connection
        let cfg = remoc::Cfg::default();
        let (conn, _sender, mut receiver) = remoc::Connect::io(cfg, socket_rx, socket_tx)
            .await
            .context("failed to initialize remoc connection")?;

        // Start remoc connection
        let connection = tokio::spawn(async move {
            let res = conn
                .instrument(info_span!("remoc connection").or_current())
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
            events,
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
        connection.abort();

        result
    }

    async fn handle_requests(&self, receiver: &mut rch::base::Receiver<BaseRequest>) -> Result<()> {
        while let Some(req) = receiver.recv().await? {
            match req {
                BaseRequest::EventBroadcast { callback } => {
                    self.handle_event_broadcast_request(callback)
                }
                BaseRequest::Cache { callback } => self.handle_cache_request(callback),
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    fn handle_cache_request(&self, callback: rch::oneshot::Sender<CacheResponse>) {
        debug!("received cache request");

        let res = CacheResponse {
            client: self.cache.clone(),
        };

        if let Err(err) = callback.send(res) {
            warn!(error = %err, "failed to send cache response")
        };
    }

    #[instrument(skip_all)]
    fn handle_event_broadcast_request(
        &self,
        callback: rch::oneshot::Sender<EventBroadcastResponse>,
    ) {
        debug!("received event broadcast request");

        let events = self.events.subscribe();
        let shutdown = self.shutdown.subscriber();

        tokio::spawn(async move {
            EventBroadcastHandler::start(events, callback, shutdown).await;
        });
    }
}
