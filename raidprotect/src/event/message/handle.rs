use std::sync::Arc;

use tracing::info;
use twilight_model::channel::Message;

use super::parser::parse_message;
use crate::cluster::ClusterState;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    let parsed = parse_message(&message);
    state.redis().set(&parsed).await.ok();

    info!("received message: {}", message.content) // Debug util real implementation
}
