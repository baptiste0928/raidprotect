use remoc::rch;
use tokio::{
    net::{TcpStream, ToSocketAddrs},
    task::JoinHandle,
};
use tracing::{error, info_span, Instrument};

use crate::model::BaseRequest;

use super::ClientError;

/// Wrapper around a raw remoc connection over TCP.
pub struct Connection {
    /// Base channel sender
    sender: rch::base::Sender<BaseRequest>,
    /// Base channel receiver (unused)
    _receiver: rch::base::Receiver<()>,
    /// Connection task handle
    ///
    /// The connection is aborted when this type is dropped.
    connection: JoinHandle<()>,
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
        let (conn, sender, _receiver) = remoc::Connect::io(cfg, socket_rx, socket_tx).await?;

        let connection = tokio::spawn(async move {
            let res = conn
                .instrument(info_span!("remoc connection").or_current())
                .await;

            if let Err(err) = res {
                error!(error = %err, "remoc connection error")
            }
        });

        Ok(Self {
            sender,
            _receiver,
            connection,
        })
    }

    /// Send a request through the connection
    pub async fn send(&mut self, item: BaseRequest) -> Result<(), ClientError> {
        self.sender.send(item).await?;

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.connection.abort()
    }
}
