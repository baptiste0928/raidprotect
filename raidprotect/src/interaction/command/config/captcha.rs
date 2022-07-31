//! Captcha configuration commands.

use anyhow::bail;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    channel::message::MessageFlags,
    http::interaction::InteractionResponseType,
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    cluster::ClusterState,
    interaction::{
        embed::{self, COLOR_GREEN, COLOR_RED},
        response::InteractionResponse,
        util::{CustomId, InteractionExt},
    },
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "captcha", desc = "Configure the RaidProtect captcha")]
pub enum CaptchaConfigCommand {
    #[command(name = "enable")]
    Enable(CaptchaEnableCommand),
    #[command(name = "disable")]
    Disable(CaptchaDisableCommand),
    #[command(name = "logs")]
    Logs(CaptchaLogsCommand),
    #[command(name = "autorole-add")]
    AutoroleAdd(CaptchaAutoroleAddCommand),
    #[command(name = "autorole-remove")]
    AutoroleRemove(CaptchaAutoroleRemoveCommand),
    #[command(name = "autorole-list")]
    AutoroleList(CaptchaAutoroleListCommand),
}

impl CaptchaConfigCommand {
    pub(super) async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        match self {
            CaptchaConfigCommand::Enable(command) => command.exec(interaction, state).await,
            CaptchaConfigCommand::Disable(command) => command.exec(interaction, state).await,
            CaptchaConfigCommand::Logs(command) => command.exec(interaction, state).await,
            CaptchaConfigCommand::AutoroleAdd(_) => todo!(),
            CaptchaConfigCommand::AutoroleRemove(_) => todo!(),
            CaptchaConfigCommand::AutoroleList(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "enable", desc = "Enable the RaidProtect captcha")]
pub struct CaptchaEnableCommand;

impl CaptchaEnableCommand {
    async fn exec(
        self,
        interaction: Interaction,
        _state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let lang = interaction.locale()?;

        let embed = EmbedBuilder::new()
            .color(COLOR_RED)
            .title(lang.captcha_enable_title())
            .description(lang.captcha_enable_description())
            .build();

        let custom_id = CustomId::name("captcha-enable");
        let components = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some(custom_id.to_string()),
                    disabled: false,
                    emoji: None,
                    label: Some(lang.captcha_enable_button().to_string()),
                    style: ButtonStyle::Success,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(lang.learn_more().to_string()),
                    style: ButtonStyle::Link,
                    url: Some("https://docs.raidprotect.org/".to_string()),
                }),
            ],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .components([components])
            .flags(MessageFlags::EPHEMERAL)
            .build();

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "disable", desc = "Disable the RaidProtect captcha")]
pub struct CaptchaDisableCommand;

impl CaptchaDisableCommand {
    async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let lang = interaction.locale()?;
        let guild_id = interaction.guild()?.id;

        let config = state.mongodb().get_guild(guild_id).await?;
        let captcha_enabled = config.as_ref().map(|c| c.captcha.enabled).unwrap_or(false);

        if !captcha_enabled {
            return Ok(embed::captcha::not_enabled(lang));
        }

        let config = config.unwrap(); // SAFETY: `captcha_enabled` is true here.
        let verification = match config.captcha.channel {
            Some(channel) => channel.mention(),
            None => bail!("captcha channel not set"),
        };
        let unverified = match config.captcha.role {
            Some(role) => role.mention(),
            None => bail!("captcha role not set"),
        };

        let embed = EmbedBuilder::new()
            .color(COLOR_RED)
            .title(lang.captcha_disable_title())
            .description(lang.captcha_disable_description(unverified, verification))
            .build();

        let custom_id = CustomId::name("captcha-disable");
        let components = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(custom_id.to_string()),
                disabled: false,
                emoji: None,
                label: Some(lang.captcha_disable_button().to_string()),
                style: ButtonStyle::Danger,
                url: None,
            })],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed])
            .components([components])
            .flags(MessageFlags::EPHEMERAL)
            .build();

        Ok(InteractionResponse::Raw {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "logs", desc = "Set the RaidProtect captcha logs channel")]
pub struct CaptchaLogsCommand {
    /// Channel to send the logs to.
    channel: Id<ChannelMarker>,
}

impl CaptchaLogsCommand {
    async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let lang = interaction.locale()?;
        let guild_id = interaction.guild()?.id;

        let config = state.mongodb().get_guild(guild_id).await?;
        let captcha_enabled = config.as_ref().map(|c| c.captcha.enabled).unwrap_or(false);

        if !captcha_enabled {
            return Ok(embed::captcha::not_enabled(lang));
        }

        // Update the config.
        let mut config = config.unwrap(); // SAFETY: `captcha_enabled` is true here
        config.captcha.logs = Some(self.channel);

        state.mongodb().update_guild(&config).await?;

        // Send the embed.
        let embed = EmbedBuilder::new()
            .color(COLOR_GREEN)
            .description(lang.captcha_logs_description(self.channel.mention()))
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-add",
    desc = "Add a role to the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleAddCommand {
    /// Role to add to the autorole.
    role: Id<RoleMarker>,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-remove",
    desc = "Remove a role from the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleRemoveCommand {
    /// Role to remove from the autorole.
    role: Id<RoleMarker>,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-list",
    desc = "List the roles of the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleListCommand;
