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
    impl_command_handle,
    interaction::{embed::COLOR_TRANSPARENT, response::InteractionResponse, util::InteractionExt},
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Need help to use RaidProtect?",
    dm_permission = true
)]

pub struct HelpCommand;

impl_command_handle!(HelpCommand);

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
            .title(lang.about_title())
            .description(lang.about_description());

        // Add components (buttons)
        let components = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.about_support().into()),
                    style: ButtonStyle::Link,
                    url: Some("https://raidpro.tk/discord".to_string()),
                }),
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.about_bot_invite().into()),
                    style: ButtonStyle::Link,
                    url: Some("https://raidpro.tk/invite".to_string()),
                }),
            ],
        });

        // Create response
        let mut response = InteractionResponseDataBuilder::new()
            .embeds([embed.validate()?.build()])
            .components([components])
            .build();

        // Add ephemeral flag to the response
        response.flags = response
            .flags
            .map(|flags| flags | MessageFlags::EPHEMERAL)
            .or(Some(MessageFlags::EPHEMERAL));

        // Send response
        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}
