use std::fmt::{self, Debug};

use remoc::rch;
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    sync::broadcast,
    task::JoinHandle,
};
use tracing::{error, info, info_span, warn, Instrument};

use crate::model::BaseRequest;

use super::{client::ConnectionUpdate, ClientError};

/// Wrapper around a raw remoc connection over TCP.
pub struct Connection {
    /// Base channel sender
    sender: rch::base::Sender<BaseRequest>,
    /// Connection task handle.
    ///
    /// This task is aborted when the current [`Connection`] is dropped.
    connection: JoinHandle<()>,
}

impl Connection {
    /// Start a new [`Connection`] to a remote server
    pub async fn start(
        addr: impl ToSocketAddrs,
        mut broadcast: broadcast::Sender<ConnectionUpdate>,
    ) -> Result<Self, ClientError> {
        // Start TCP connection
        let (socket_rx, socket_tx) = match TcpStream::connect(addr).await {
            Ok(socket) => socket.into_split(),
            Err(source) => return Err(ClientError::Connect { source }),
        };

        // Start remoc connection
        let cfg = remoc::Cfg::default();
        let (conn, sender, _recv): (_, _, rch::base::Receiver<()>) =
            remoc::Connect::io(cfg, socket_rx, socket_tx).await?;

        // Notify that the connection has started
        let _ = broadcast.send(ConnectionUpdate::Connected);
        info!("gateway client connected");

        let connection = tokio::spawn(async move {
            let res = conn
                .instrument(info_span!("remoc connection").or_current())
                .await;

            if let Err(err) = res {
                warn!(error = %err, "remoc connection error")
            }

            // Notify that the connection has stopped
            let _ = broadcast.send(ConnectionUpdate::Disconnected);
            warn!("gateway client disconnected")
        });

        Ok(Self { sender, connection })
    }

    /// Send a request through the connection
    pub async fn send(&mut self, req: BaseRequest) -> Result<(), ClientError> {
        self.sender.send(req).await?;

        Ok(())
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Connection { .. }")
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.connection.abort()
    }
}
