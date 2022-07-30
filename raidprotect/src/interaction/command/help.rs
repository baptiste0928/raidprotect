//! Help command.
//!
//! This command shows basic information and link about how to use the bot.

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    channel::message::MessageFlags,
    http::interaction::InteractionResponseType,
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    cluster::ClusterState,
    desc_translation, impl_command_handle,
    interaction::{embed::COLOR_TRANSPARENT, response::InteractionResponse, util::InteractionExt},
    translations::Lang,
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Need help to use RaidProtect?",
    desc_localizations = "help_description",
    dm_permission = true
)]
pub struct HelpCommand;

impl_command_handle!(HelpCommand);
desc_translation!(help_description);

impl HelpCommand {
    async fn exec(
        self,
        interaction: Interaction,
        _state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let lang = interaction.locale()?;

        // Create embed
        let embed = EmbedBuilder::new()
            .color(COLOR_TRANSPARENT)
            .title(lang.help_embed_title())
            .description(lang.help_embed_description());

        // Add components (buttons)
        let components = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.help_support().into()),
                    style: ButtonStyle::Link,
                    url: Some("https://raidpro.tk/discord".to_string()),
                }),
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.help_bot_invite().into()),
                    style: ButtonStyle::Link,
                    url: Some("https://raidpro.tk/invite".to_string()),
                }),
            ],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed.build()])
            .components([components])
            .flags(MessageFlags::EPHEMERAL)
            .build();

        // Send response
        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}
