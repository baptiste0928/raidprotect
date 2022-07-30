use std::sync::Arc;

use anyhow::{Context, Result};
use rosetta_i18n::Language;
use tracing::{error, info};
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_util::builder::embed::EmbedBuilder;

use crate::cluster::ClusterState;
use crate::interaction::embed::COLOR_TRANSPARENT;
use crate::translations::Lang;

use super::parser::parse_message;

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    let parsed = parse_message(&message);
    state.redis().set(&parsed).await.ok();

    if is_an_old_command(&message) {
        if let Err(error) = warn_about_old_command(&message, state.http()).await {
            error!(error = ?error, "failed to warn user about question mark commands deprecation");
        }
    }

    info!("received message: {}", message.content) // Debug util real implementation
}

async fn warn_old_command(message: Message, state: Arc<ClusterState> {
    let lang = Lang::fallback();
    let embeds = [EmbedBuilder::new()
        .title(lang.warning_deprecated_command_style_title())
        .description(lang.warning_deprecated_command_style_description())
        .color(COLOR_TRANSPARENT)
        .build()];
    http.create_message(message.channel_id)
        .embeds(&embeds)?
        .reply(message.id)
        .exec()
        .await
        .context("warning about old commands with question  mark ?")?;
    Ok(())
}

fn is_an_old_command(message: &Message) -> bool {
    message.content.starts_with("?kick")
        || message.content.starts_with("?userinfo")
        || message.content.starts_with("?lock")
        || message.content.starts_with("?unlock")
        || message.content.starts_with("?raidmode")
        || message.content.starts_with("?settings")
        || message.content.starts_with("?captcha")
        || message.content.starts_with("?ban")
        || message.content.starts_with("?invite")
        || message.content.starts_with("?clear")
        || message.content.starts_with("?stats")
}
