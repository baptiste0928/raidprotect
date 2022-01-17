use std::{
    error::Error,
    fmt::{self, Display},
    io,
    net::IpAddr,
};

/// [`remoc`] connection error.
pub type RemocConnectError = remoc::ConnectError<io::Error, io::Error>;

/// An error occurred with the [`GatewayClient`].
///
/// [`GatewayClient`]: super::GatewayClient
#[derive(Debug)]
pub enum ClientError {
    /// Failed to connect to the server.
    Connect { source: io::Error },
    /// Failed to intialize [`remoc`] connection.
    RemocConnect { source: RemocConnectError },
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::Connect { source } => Some(source),
            ClientError::RemocConnect { source } => Some(source),
            _ => None,
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Connect { source } => {
                write!(f, "failed to connect to remote server: {}", source)
            }
            ClientError::RemocConnect { source } => {
                write!(f, "failed to intialize remoc connection: {}", source)
            }
        }
    }
}

impl From<RemocConnectError> for ClientError {
    fn from(source: RemocConnectError) -> Self {
        Self::RemocConnect { source }
    }
}
