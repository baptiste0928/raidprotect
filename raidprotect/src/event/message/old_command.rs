use std::collections::HashMap;

use anyhow::bail;
use once_cell::sync::Lazy;
use twilight_model::channel::Message;
use twilight_util::builder::embed::EmbedBuilder;

use crate::{interaction::embed::COLOR_TRANSPARENT, shard::BotState, translations::Lang};

/// Mapping of old command names to new command names.
static OLD_COMMANDS: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("?about", "/help"),
        ("?ban", "/ban"),
        ("?captcha", "/config captcha"),
        ("?clear", "/clear"),
        ("?help", "/help"),
        ("?invite", "/help"),
        ("?kick", "/kick"),
        ("?ping", "/ping"),
        ("?raidmode", "/raidmode"),
        ("?settings", "/config"),
        ("?stats", "/help"),
        ("?ui", "/profile"),
        ("?userinfo", "/profile"),
    ])
});

/// Check whether a message contains an old command.
pub fn is_old_command(content: &str) -> bool {
    if let Some((command, _)) = content.split_once(' ') {
        return OLD_COMMANDS.contains_key(command);
    }

    false
}

/// Send a warning message to the user that they used an old command.
pub async fn warn_old_command(message: Message, state: &BotState) -> Result<(), anyhow::Error> {
    let lang = message
        .author
        .locale
        .map(|lang| Lang::from(&*lang))
        .unwrap_or(Lang::DEFAULT);

    if let Some((command, _)) = message.content.split_once(' ') {
        let new = match OLD_COMMANDS.get(command) {
            Some(new) => new,
            None => bail!("no command matching {} found", command),
        };

        let embed = EmbedBuilder::new()
            .title(lang.warning_deprecated_command_title())
            .description(lang.warning_deprecated_command_description(new, command))
            .color(COLOR_TRANSPARENT)
            .build();

        state
            .http
            .create_message(message.channel_id)
            .reply(message.id)
            .embeds(&[embed])?
            .exec()
            .await?;
    }

    Ok(())
}
