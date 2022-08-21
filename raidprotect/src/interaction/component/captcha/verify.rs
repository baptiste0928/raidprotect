//! Captcha verification button and modal.

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use raidprotect_captcha::{code::random_human_code, generate_captcha_png};
use raidprotect_model::cache::model::interaction::PendingCaptcha;
use tracing::{error, instrument};
use twilight_http::request::AuditLogReason;
use twilight_model::{
    application::{
        component::{
            button::ButtonStyle, text_input::TextInputStyle, ActionRow, Button, Component,
            TextInput,
        },
        interaction::Interaction,
    },
    channel::message::MessageFlags,
    http::{attachment::Attachment, interaction::InteractionResponseType},
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};
use twilight_util::builder::{
    embed::{EmbedBuilder, ImageSource},
    InteractionResponseDataBuilder,
};

use crate::{
    cluster::ClusterState,
    feature::captcha,
    interaction::{
        embed::{self, COLOR_TRANSPARENT},
        response::InteractionResponse,
        util::{CustomId, InteractionExt},
    },
    translations::Lang,
};

/// Captcha verification button.
///
/// This button is used to send the verification message to a user along with
/// a generated captcha image.
pub struct CaptchaVerifyButton;

impl CaptchaVerifyButton {
    #[instrument(skip(state))]
    pub async fn handle(
        interaction: Interaction,
        state: Arc<ClusterState>,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let guild = interaction.guild()?;
        let author = interaction.author_id().context("missing author_id")?;
        let lang = interaction.locale()?;

        let config = state.mongodb().get_guild_or_create(guild.id).await?;
        let guild_lang = Lang::from(&*config.lang);

        // Get the pending captcha from the cache.
        let mut captcha = match state
            .redis()
            .get::<PendingCaptcha>(&(guild.id, author))
            .await?
        {
            Some(captcha) => captcha,
            None => {
                return Ok(embed::captcha::captcha_not_found(lang));
            }
        };

        // Captcha has been regenerated too many times.
        if captcha.regenerate_count >= captcha::MAX_RETRY {
            tokio::spawn(kick_after(
                state,
                guild.id,
                author,
                captcha::KICK_AFTER,
                guild_lang,
            ));

            return Ok(embed::captcha::regenerate_error(lang));
        }

        // Generate the captcha code.
        let code = random_human_code(captcha::DEFAULT_LENGTH);

        let code_clone = code.clone();
        let image =
            tokio::task::spawn_blocking(move || generate_captcha_png(&code_clone)).await??;

        // Update the captcha in the cache.
        captcha.code = code;
        captcha.regenerate_count += 1;

        state.redis().set(&captcha).await?;

        // Send the verification message.
        let embed = EmbedBuilder::new()
            .title(lang.captcha_image_title())
            .color(COLOR_TRANSPARENT)
            .description(lang.captcha_image_description())
            .image(ImageSource::attachment("captcha.png")?)
            .build();

        let custom_id = CustomId::name("captcha-validate");
        let component = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(custom_id.to_string()),
                label: Some(lang.captcha_image_button().to_string()),
                style: ButtonStyle::Success,
                disabled: false,
                emoji: None,
                url: None,
            })],
        });

        let attachment = Attachment {
            file: image,
            filename: "captcha.png".to_string(),
            id: 0,
            description: Some(lang.captcha_image_alt().to_string()),
        };

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .components([component])
            .attachments([attachment])
            .flags(MessageFlags::EPHEMERAL)
            .build();

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}

async fn kick_after(
    state: Arc<ClusterState>,
    guild: Id<GuildMarker>,
    user: Id<UserMarker>,
    after: Duration,
    lang: Lang,
) {
    tokio::time::sleep(after).await;

    let http = state.cache_http(guild);
    let req = match http.remove_guild_member(user).await {
        Ok(req) => req,
        Err(error) => {
            error!(error = ?error, "missing permissions to kick user after captcha");
            return;
        }
    };

    if let Err(error) = req.reason(lang.captcha_kick_reason()).unwrap().exec().await {
        error!(error = ?error, "failed to kick user after captcha");
    }
}

/// Captcha validation button.
///
/// This button send the captcha modal to the user.
pub struct CaptchaValidateButton;

impl CaptchaValidateButton {
    #[instrument(skip(state))]
    pub async fn handle(
        interaction: Interaction,
        state: Arc<ClusterState>,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let guild = interaction.guild()?;
        let author = interaction.author_id().context("missing author_id")?;
        let lang = interaction.locale()?;

        // Ensure the user is not already verified.
        match state
            .redis()
            .get::<PendingCaptcha>(&(guild.id, author))
            .await?
        {
            Some(captcha) => captcha,
            None => {
                return Ok(embed::captcha::captcha_not_found(lang));
            }
        };

        // Send the captcha modal.
        let input_custom_id = CustomId::name("captcha-input");
        let modal_custom_id = CustomId::name("captcha-modal");

        let components = vec![Component::ActionRow(ActionRow {
            components: vec![Component::TextInput(TextInput {
                custom_id: input_custom_id.to_string(),
                label: lang.captcha_input_label().to_string(),
                max_length: Some(captcha::DEFAULT_LENGTH as u16),
                min_length: Some(captcha::DEFAULT_LENGTH as u16),
                placeholder: Some("-".repeat(captcha::DEFAULT_LENGTH)),
                required: Some(true),
                style: TextInputStyle::Short,
                value: None,
            })],
        })];

        Ok(InteractionResponse::Modal {
            custom_id: modal_custom_id.to_string(),
            title: lang.captcha_image_title().to_string(),
            components,
        })
    }
}
