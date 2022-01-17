use std::sync::Arc;

use tokio::sync::RwLock;

use super::connect::Connection;

/// Gateway client.
///
/// The internal client state is held in an [`Arc`], allowing to cheaply clone
/// this type.
pub struct GatewayClient {
    inner: Arc<RwLock<ClientInner>>,
}

/// Internal client state.
pub struct ClientInner {
    connection: Connection,
}
