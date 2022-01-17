use std::net::IpAddr;

use anyhow::{Result, Context};
use tokio::net::TcpStream;

/// Wrapper around a raw remoc connection over TCP
pub struct Connection {}

impl Connection {
    /// Start a new [`Connection`] to a remote server
    pub async fn new(addr: IpAddr, port: u16) -> Result<Self> {
        let socket = TcpStream::connect((addr, port))
            .await
            .context(format!("Failed to connect to {addr}:{port}"));

        todo!()
    }
}
