use std::{
    error::Error,
    fmt::{self, Display},
    io,
};

use remoc::rch;

use crate::{cache::CacheError, model::BaseRequest};

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
    /// Request timed out
    Timeout,
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClientError::Connect { source } => Some(source),
            ClientError::RemocConnect { source } => Some(source),
            ClientError::BaseSend { source } => Some(source),
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
                write!(f, "failed to initialize remoc connection: {}", source)
            }
            ClientError::BaseSend { source } => {
                write!(f, "failed to send request through base channel: {}", source)
            }
            ClientError::Timeout => f.write_str("request timed out"),
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

impl From<ReconnectTimeoutError> for ClientError {
    fn from(_: ReconnectTimeoutError) -> Self {
        Self::Timeout
    }
}

/// Automatic reconnection to the server timed out.
#[derive(Debug)]
pub struct ReconnectTimeoutError;

impl Error for ReconnectTimeoutError {}

impl Display for ReconnectTimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("automatic reconnection timed out")
    }
}

impl From<ReconnectTimeoutError> for CacheError {
    fn from(_: ReconnectTimeoutError) -> Self {
        CacheError::ReconnectTimeout
    }
}
