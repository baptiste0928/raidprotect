use std::sync::Arc;

use raidprotect_analyzer::parse_message;
use raidprotect_model::ClusterState;
use twilight_model::channel::Message;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    state.messages().insert(parse_message(&message)).await;

    println!("received message: {}", message.content) // Debug util real implementation
}
