//! Captcha disable button.

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
        embed::{self, COLOR_RED, COLOR_SUCCESS},
        response::InteractionResponse,
        util::{GuildConfigExt, GuildInteractionContext},
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
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let ctx = GuildInteractionContext::new(interaction)?;
        let mut config = ctx.config(state).await?;
        let guild_lang = config.lang();

        // Ensure the captcha is enabled.
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

        // Try to delete the verification channel and the unverified role.
        if let Some(role) = config.captcha.role {
            if let Err(error) = state
                .http
                .delete_role(ctx.guild_id, role)
                .reason(guild_lang.captcha_disable_reason())?
                .exec()
                .await
            {
                warn!(error = ?error, guild = ?ctx.guild_id, role = ?role, "failed to delete unverified role");
            }
        }

        if let Some(channel) = config.captcha.channel {
            if let Err(error) = state
                .http
                .delete_channel(channel)
                .reason(guild_lang.captcha_disable_reason())?
                .exec()
                .await
            {
                warn!(error = ?error, guild = ?ctx.guild_id, channel = ?channel, "failed to delete verification channel");
            }
        }

        // Update the configuration.
        config.captcha = Default::default();
        state.database.update_guild(&config).await?;

        // Send message in logs channel.
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(error) = logs_message(
                &state_clone,
                ctx.guild_id,
                config.logs_chan,
                ctx.author.id,
                guild_lang,
            )
            .await
            {
                error!(error = ?error, guild = ?ctx.guild_id, "failed to send captcha disable logs message");
            }
        });

        // Send the confirmation message.
        let embed = EmbedBuilder::new()
            .title(ctx.lang.captcha_disabled_title())
            .color(COLOR_SUCCESS)
            .description(ctx.lang.captcha_disabled_description())
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

async fn logs_message(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    logs_channel: Option<Id<ChannelMarker>>,
    user: Id<UserMarker>,
    lang: Lang,
) -> Result<(), anyhow::Error> {
    let channel = guild_logs_channel(state, guild, logs_channel, lang).await?;

    let embed = EmbedBuilder::new()
        .color(COLOR_RED)
        .description(lang.captcha_disabled_log(user.mention()))
        .build();

    state
        .http
        .create_message(channel)
        .embeds(&[embed])?
        .exec()
        .await?;

    Ok(())
}
