use std::sync::Arc;

use raidprotect_state::ClusterState;
use twilight_model::channel::Message;

use super::parser::parse_message;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    state.messages().insert(parse_message(&message)).await;

    println!("received message: {}", message.content) // Debug util real implementation
}
