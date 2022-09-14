use anyhow::Context;
use tracing::{error, info};
use twilight_model::{channel::Message, gateway::payload::incoming::MessageDelete};

use super::{
    old_command::{is_old_command, warn_old_command},
    parser::parse_message,
};
use crate::cluster::ClusterState;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message_create(message: Message, state: &ClusterState) {
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

/// Handle deleted [`Message`].
pub async fn handle_message_delete(event: MessageDelete, state: &ClusterState) {
    if let Err(error) = handle_message_delete_inner(event, state).await {
        error!(error = ?error, "error while handle message delete");
    }
}

async fn handle_message_delete_inner(
    event: MessageDelete,
    state: &ClusterState,
) -> Result<(), anyhow::Error> {
    let guild_id = event
        .guild_id
        .context("missing guild_id in message delete event")?;

    let config = state
        .database
        .get_guild_or_create(guild_id)
        .await
        .context("failed to get guild configuration")?;

    // Resend the captcha message if deleted.
    if Some(event.id) == config.captcha.message {}

    Ok(())
}
