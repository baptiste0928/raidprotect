//! Captcha verification button and modal.

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
        util::{CustomId, GuildConfigExt, GuildInteractionContext},
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
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let ctx = GuildInteractionContext::new(interaction)?;

        // Get the pending captcha from the cache.
        let mut captcha = match get_captcha(&ctx, state).await? {
            Some(captcha) => captcha,
            None => {
                return Ok(embed::captcha::captcha_not_found(ctx.lang));
            }
        };

        // Kick the user if the captcha has been regenerated too many times.
        if captcha.regenerate_count >= captcha::MAX_RETRY {
            let config = ctx.config(state).await?;
            let state_clone = state.clone();

            tokio::spawn(async move {
                kick_after(&state_clone, ctx.guild_id, ctx.author.id, config.lang()).await
            });

            return Ok(embed::captcha::regenerate_error(ctx.lang));
        }

        // Generate the captcha image.
        let code = random_human_code(captcha::DEFAULT_LENGTH);

        let code_clone = code.clone();
        let image =
            tokio::task::spawn_blocking(move || generate_captcha_png(&code_clone)).await??;

        // Update the captcha in the cache.
        captcha.code = code;
        captcha.regenerate_count += 1;

        state.cache.set(&captcha).await?;

        // Send the verification message.
        let embed = EmbedBuilder::new()
            .title(ctx.lang.captcha_image_title())
            .color(COLOR_TRANSPARENT)
            .description(ctx.lang.captcha_image_description())
            .image(ImageSource::attachment("captcha.png")?)
            .build();

        let continue_id = CustomId::name("captcha-validate");
        let mut components = vec![Component::Button(Button {
            custom_id: Some(continue_id.to_string()),
            label: Some(ctx.lang.captcha_image_button().to_owned()),
            style: ButtonStyle::Success,
            disabled: false,
            emoji: None,
            url: None,
        })];

        // Add regenerate button if MAX_RETRY is not reached.
        // The button will re-trigger the current interaction.
        if captcha.regenerate_count < captcha::MAX_RETRY {
            let regenerate_id = CustomId::name("captcha-verify");
            components.push(Component::Button(Button {
                custom_id: Some(regenerate_id.to_string()),
                label: Some(ctx.lang.captcha_image_regenerate().to_owned()),
                style: ButtonStyle::Secondary,
                disabled: false,
                emoji: None,
                url: None,
            }));
        }

        let component = Component::ActionRow(ActionRow { components });
        let attachment = Attachment {
            file: image,
            filename: "captcha.png".to_owned(),
            id: 0,
            description: Some(ctx.lang.captcha_image_alt().to_owned()),
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

/// Kick user that failed to verify after 10 seconds.
pub async fn kick_after(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    user: Id<UserMarker>,
    guild_lang: Lang,
) {
    tokio::time::sleep(captcha::KICK_AFTER).await;

    let http = state.cache_http(guild);
    let req = match http.remove_guild_member(user).await {
        Ok(req) => req,
        Err(error) => {
            error!(error = ?error, "missing permissions to kick user after captcha");
            return;
        }
    };

    if let Err(error) = req
        .reason(guild_lang.captcha_kick_reason())
        .unwrap()
        .exec()
        .await
    {
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
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let ctx = GuildInteractionContext::new(interaction)?;

        // Get the captcha code length from the cache.
        let code_length = match get_captcha(&ctx, state).await? {
            Some(captcha) => captcha.code.len(),
            None => {
                return Ok(embed::captcha::captcha_not_found(ctx.lang));
            }
        };

        // Send the captcha modal.
        let input_custom_id = CustomId::name("captcha-input");
        let modal_custom_id = CustomId::name("captcha-modal");

        let components = vec![Component::ActionRow(ActionRow {
            components: vec![Component::TextInput(TextInput {
                custom_id: input_custom_id.to_string(),
                label: ctx.lang.captcha_input_label().to_owned(),
                max_length: Some(code_length as u16),
                min_length: Some(code_length as u16),
                placeholder: Some("-".repeat(code_length)),
                required: Some(true),
                style: TextInputStyle::Short,
                value: None,
            })],
        })];

        Ok(InteractionResponse::Modal {
            custom_id: modal_custom_id.to_string(),
            title: ctx.lang.captcha_image_title().to_owned(),
            components,
        })
    }
}

/// Get the captcha key from the current context.
pub fn captcha_key(ctx: &GuildInteractionContext) -> (Id<GuildMarker>, Id<UserMarker>) {
    (ctx.guild_id, ctx.author.id)
}

/// Get the pending captcha from the cache.
pub async fn get_captcha(
    ctx: &GuildInteractionContext,
    state: &ClusterState,
) -> Result<Option<PendingCaptcha>, anyhow::Error> {
    state.cache.get::<PendingCaptcha>(&captcha_key(ctx)).await
}
