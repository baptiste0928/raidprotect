use std::{
    error::Error,
    fmt::{self, Display},
    io,
    net::IpAddr,
};

/// An error occurred with the [`GatewayClient`].
///
/// [`GatewayClient`]: super::GatewayClient
#[derive(Debug)]
pub enum ClientError {
    /// Failed to connect to the server
    Connect {
        addr: (IpAddr, u16),
        source: io::Error,
    },
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::Connect { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Connect { addr, source } => {
                write!(f, "failed to connect to {}:{}: {}", addr.0, addr.1, source)
            }
        }
    }
}
