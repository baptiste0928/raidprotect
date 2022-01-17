use std::net::IpAddr;

use remoc::rch;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::{error, info_span, Instrument};

use super::ClientError;

/// Wrapper around a raw remoc connection over TCP
pub struct Connection {
    tx: rch::base::Sender<()>,
    rx: rch::base::Receiver<()>,
}

impl Connection {
    /// Start a new [`Connection`] to a remote server
    pub async fn start(addr: impl ToSocketAddrs) -> Result<Self, ClientError> {
        // Start TCP connection
        let (socket_rx, socket_tx) = match TcpStream::connect(addr).await {
            Ok(socket) => socket.into_split(),
            Err(source) => return Err(ClientError::Connect { source }),
        };

        // Start remoc connection
        let cfg = remoc::Cfg::default();
        let (conn, tx, rx) = remoc::Connect::io(cfg, socket_rx, socket_tx).await?;

        let connection = tokio::spawn(async move {
            let res = conn
                .instrument(info_span!("remoc connection").or_current())
                .await;

            if let Err(err) = res {
                error!(error = %err, "remoc connection error")
            }
        });

        Ok(Self { tx, rx })
    }
}
