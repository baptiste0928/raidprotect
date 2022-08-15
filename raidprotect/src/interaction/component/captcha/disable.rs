//! Captcha disable button.

use std::sync::Arc;

use anyhow::Context;
use tracing::{error, warn};
use twilight_http::request::AuditLogReason;
use twilight_mention::Mention;
use twilight_model::{
    application::interaction::Interaction,
    id::{
        marker::{ChannelMarker, GuildMarker, UserMarker},
        Id,
    },
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    cluster::ClusterState,
    interaction::{
        embed::{self, COLOR_GREEN, COLOR_RED},
        response::InteractionResponse,
        util::InteractionExt,
    },
    translations::Lang,
    util::guild_logs_channel,
};

/// Captcha disable button.
///
/// This type handle the button used to disable the captcha (sent by the
/// `/config captcha disable`) command.
///
/// It will disable the captcha from the guild configuration and try to delete
/// the verification channel and the unverified role.
pub struct CaptchaDisable;

impl CaptchaDisable {
    pub async fn handle(
        interaction: Interaction,
        state: Arc<ClusterState>,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let guild = interaction.guild()?;
        let lang = interaction.locale()?;
        let author_id = interaction.author_id().context("missing author id")?;

        let mut config = state.mongodb().get_guild_or_create(guild.id).await?;
        let guild_lang = Lang::from(&*config.lang);

        // Ensure the captcha is enabled.
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(lang));
        }

        // Try to delete the verification channel and the unverified role.
        if let Some(role) = config.captcha.role {
            if let Err(error) = state
                .http()
                .delete_role(guild.id, role)
                .reason(guild_lang.captcha_disable_reason())?
                .exec()
                .await
            {
                warn!(error = ?error, guild = ?guild.id, role = ?role, "failed to delete unverified role");
            }
        }

        if let Some(channel) = config.captcha.channel {
            if let Err(error) = state
                .http()
                .delete_channel(channel)
                .reason(guild_lang.captcha_disable_reason())?
                .exec()
                .await
            {
                warn!(error = ?error, guild = ?guild.id, channel = ?channel, "failed to delete verification channel");
            }
        }

        // Update the configuration.
        config.captcha = Default::default();
        state.mongodb().update_guild(&config).await?;

        // Send message in logs channel.
        tokio::spawn(async move {
            if let Err(error) =
                logs_message(state, guild.id, config.logs_chan, author_id, guild_lang).await
            {
                error!(error = ?error, guild = ?guild.id, "failed to send captcha disable logs message");
            }
        });

        // Send the confirmation message.
        let embed = EmbedBuilder::new()
            .title(lang.captcha_disabled_title())
            .color(COLOR_GREEN)
            .description(lang.captcha_disabled_description())
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

async fn logs_message(
    state: Arc<ClusterState>,
    guild: Id<GuildMarker>,
    logs_channel: Option<Id<ChannelMarker>>,
    user: Id<UserMarker>,
    lang: Lang,
) -> Result<(), anyhow::Error> {
    let channel = guild_logs_channel(&state, guild, logs_channel, lang).await?;

    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_disabled_log(user.mention()))
        .build();

    state
        .http()
        .create_message(channel)
        .embeds(&[embed])?
        .exec()
        .await?;

    Ok(())
}
