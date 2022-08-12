use std::{collections::HashMap, sync::Arc};

use once_cell::sync::Lazy;
use tracing::{error, info};
use twilight_model::channel::Message;
use twilight_util::builder::embed::EmbedBuilder;

use super::parser::parse_message;
use crate::{cluster::ClusterState, interaction::embed::COLOR_TRANSPARENT, translations::Lang};

/// A mapping between old and new commands
static OLD_COMMANDS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("?kick", "/kick"),
        ("?ban", "/ban"),
        ("?clear", "/clear"),
        ("?raidmode", "/raidmode"),
        ("?settings", "/config"),
        ("?captcha", "/config captcha"),
        ("?userinfo", "/profile"),
        ("?ui", "/profile"),
        ("?help", "/help"),
        ("?invite", "/help"),
        ("?about", "/help"),
        ("?stats", "/help"),
        ("?ping", "/ping"),
    ])
});

/// Handle incoming [`Message`].
///
/// This method will forward message to the cache and various auto-moderation
/// modules.
pub async fn handle_message(message: Message, state: Arc<ClusterState>) {
    let parsed = parse_message(&message);
    state.redis().set(&parsed).await.ok();

    if is_old_command(&message.content) {
        let message = message.clone();
        let state = state.clone();

        tokio::spawn(warn_old_command(message, state));
    }

    info!("received message: {}", message.content) // Debug util real implementation
}

async fn warn_old_command(message: Message, state: Arc<ClusterState>) {
    let lang = message
        .author
        .locale
        .map(|locale| Lang::from(locale.as_str()))
        .unwrap_or(Lang::DEFAULT);

    let (old_cmd, new_cmd) = match OLD_COMMANDS
        .iter()
        .find(|(old_cmd, _)| message.content.starts_with(**old_cmd))
    {
        Some((old_cmd, new_cmd)) => (*old_cmd, *new_cmd),
        _ => ("", ""),
    };

    let embed = EmbedBuilder::new()
        .title(lang.warning_deprecated_command_title())
        .description(lang.warning_deprecated_command_description(new_cmd, old_cmd))
        .color(COLOR_TRANSPARENT)
        .build();

    match state
        .http()
        .create_message(message.channel_id)
        .reply(message.id)
        .embeds(&[embed])
    {
        Ok(msg) => {
            if let Err(error) = msg.exec().await {
                error!(error = ?error, "failed to send command deprecation warning");
            }
        }
        Err(error) => error!(error = ?error, "failed to create embed"),
    }
}

fn is_old_command(content: &str) -> bool {
    OLD_COMMANDS.keys().any(|cmd| content.starts_with(cmd))
}
