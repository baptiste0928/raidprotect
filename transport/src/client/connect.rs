use std::net::IpAddr;

use tokio::net::TcpStream;

use super::ClientError;

/// Wrapper around a raw remoc connection over TCP
pub struct Connection {}

impl Connection {
    /// Start a new [`Connection`] to a remote server
    pub async fn new(addr: IpAddr, port: u16) -> Result<Self, ClientError> {
        // Start TCP connection
        let socket =
            TcpStream::connect((addr, port))
                .await
                .map_err(|error| ClientError::Connect {
                    addr: (addr, port),
                    source: error,
                })?;

        let (read, write) = socket.into_split();

        todo!()
    }
}
