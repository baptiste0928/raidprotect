//! Captcha verification button and modal.

use anyhow::Context;
use raidprotect_captcha::{code::random_human_code, generate_captcha_png};
use raidprotect_model::cache::model::interaction::PendingCaptcha;
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    channel::message::MessageFlags,
    http::{attachment::Attachment, interaction::InteractionResponseType},
};
use twilight_util::builder::{
    embed::{EmbedBuilder, ImageSource},
    InteractionResponseDataBuilder,
};

use crate::{
    cluster::ClusterState,
    interaction::{
        embed::{COLOR_TRANSPARENT},
        response::InteractionResponse, util::InteractionExt,
    },
};

/// Captcha verification button.
///
/// This button is used to send the verification message to a user along with
/// a generated captcha image.
pub struct CaptchaVerifyButton;

impl CaptchaVerifyButton {
    pub async fn handle(
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let guild = interaction.guild()?;
        let author = interaction.author_id().context("missing author_id")?;

        // Get the pending captcha from the state.
        let _captcha = state
            .redis()
            .get::<PendingCaptcha>(&(guild.id, author))
            .await?;

        // TODO ...

        // Generate the captcha image.
        let code = random_human_code(5);
        let image = tokio::task::spawn_blocking(move || generate_captcha_png(&code)).await??;

        // Send the verification message.
        let embed = EmbedBuilder::new()
            .title("Complétez le captcha pour continuer")
            .color(COLOR_TRANSPARENT)
            .description("Pour accéder au serveur, __mémorisez le code que vous lisez dans l'image ci-dessous__ puis cliquez sur le bouton et écrivez-le dans le formulaire qui s'affichera.\n\nSi vous ne validez pas ce captcha, vous serez expulsé du serveur dans 5 minutes. Vous pouvez le regénérer si vous avez des difficultés à le lire.")
            .image(ImageSource::attachment("captcha.png")?)
            .build();

        let component = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some("captcha-validate".to_string()),
                    label: Some("Continuer (entrer le code)".to_string()),
                    style: ButtonStyle::Success,
                    disabled: false,
                    emoji: None,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: Some("captcha-regenerate".to_string()),
                    label: Some("Regénérer le captcha".to_string()),
                    style: ButtonStyle::Secondary,
                    disabled: false,
                    emoji: None,
                    url: None,
                }),
            ],
        });

        let attachment = Attachment {
            file: image,
            filename: "captcha.png".to_string(),
            id: 0,
            description: Some("Captcha image".to_string()),
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
