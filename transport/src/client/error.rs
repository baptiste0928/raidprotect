use std::{
    error::Error,
    fmt::{self, Display},
    io,
};

use remoc::rch;

use crate::model::BaseRequest;

/// [`remoc`] connection error.
pub type RemocConnectError = remoc::ConnectError<io::Error, io::Error>;

/// An error occurred with the [`GatewayClient`].
///
/// [`GatewayClient`]: super::GatewayClient
#[derive(Debug)]
pub enum ClientError {
    /// Failed to connect to the server.
    Connect { source: io::Error },
    /// Failed to initialize [`remoc`] connection.
    RemocConnect { source: RemocConnectError },
    /// Error while sending date through a [`remoc::rch::base`] channel.
    BaseSend {
        source: rch::base::SendError<BaseRequest>,
    },
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::Connect { source } => Some(source),
            ClientError::RemocConnect { source } => Some(source),
            ClientError::BaseSend { source } => Some(source),
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
                write!(f, "failed to initialize remoc connection: {}", source)
            }
            ClientError::BaseSend { source } => {
                write!(f, "failed to send request through base channel: {}", source)
            }
        }
    }
}

impl From<RemocConnectError> for ClientError {
    fn from(source: RemocConnectError) -> Self {
        Self::RemocConnect { source }
    }
}

impl From<rch::base::SendError<BaseRequest>> for ClientError {
    fn from(source: rch::base::SendError<BaseRequest>) -> Self {
        Self::BaseSend { source }
    }
}
