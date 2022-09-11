use tracing::{error, info};
use twilight_model::channel::Message;

use super::{
    old_command::{is_old_command, warn_old_command},
    parser::parse_message,
};
use crate::cluster::ClusterState;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: &ClusterState) {
    // Ignore messages from bots.
    if message.author.bot {
        return;
    }

    let parsed = parse_message(&message);
    state.cache.set(&parsed).await.ok();

    // Warn the user if they're using an old command.
    if is_old_command(&message.content) {
        let (message, state) = (message.clone(), state.clone());

        tokio::spawn(async move {
            if let Err(error) = warn_old_command(message, &state).await {
                error!(error = ?error, "failed to warn user about old command");
            }
        });
    }

    info!("received message: {}", message.content) // Debug util real implementation
}
