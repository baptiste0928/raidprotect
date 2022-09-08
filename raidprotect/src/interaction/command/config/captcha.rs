//! Captcha configuration commands.

use anyhow::bail;
use raidprotect_model::{cache::discord::permission::RoleOrdering, database::model::CaptchaConfig};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_mention::Mention;
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::message::MessageFlags,
    guild::{Permissions, Role},
    http::interaction::InteractionResponseType,
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    cluster::ClusterState,
    desc_localizations,
    interaction::{
        embed::{self, COLOR_GREEN, COLOR_RED, COLOR_TRANSPARENT},
        response::InteractionResponse,
        util::{CustomId, GuildInteractionContext},
    },
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "captcha",
    desc = "Configure the RaidProtect captcha",
    desc_localizations = "captcha_description"
)]
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

desc_localizations!(captcha_description);

impl CaptchaConfigCommand {
    pub(super) async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        match self {
            CaptchaConfigCommand::Enable(command) => command.exec(ctx, state).await,
            CaptchaConfigCommand::Disable(command) => command.exec(ctx, state).await,
            CaptchaConfigCommand::Logs(command) => command.exec(ctx, state).await,
            CaptchaConfigCommand::AutoroleAdd(command) => command.exec(ctx, state).await,
            CaptchaConfigCommand::AutoroleRemove(command) => command.exec(ctx, state).await,
            CaptchaConfigCommand::AutoroleList(command) => command.exec(ctx, state).await,
        }
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "enable",
    desc = "Enable the RaidProtect captcha",
    desc_localizations = "captcha_enable_description"
)]
pub struct CaptchaEnableCommand;

desc_localizations!(captcha_enable_description);

impl CaptchaEnableCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let config = ctx.config(state).await?;
        if config.captcha.enabled {
            return Ok(embed::captcha::already_enabled(ctx.lang));
        }

        let embed = EmbedBuilder::new()
            .color(COLOR_RED)
            .title(ctx.lang.captcha_confirm_title())
            .description(ctx.lang.captcha_confirm_description())
            .build();

        let custom_id = CustomId::name("captcha-enable");
        let components = Component::ActionRow(ActionRow {
            components: vec![
                Component::Button(Button {
                    custom_id: Some(custom_id.to_string()),
                    disabled: false,
                    emoji: None,
                    label: Some(ctx.lang.captcha_confirm_button().to_string()),
                    style: ButtonStyle::Success,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: None,
                    disabled: false,
                    emoji: None,
                    label: Some(ctx.lang.learn_more().to_string()),
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
#[command(
    name = "disable",
    desc = "Disable the RaidProtect captcha",
    desc_localizations = "captcha_disable_description"
)]
pub struct CaptchaDisableCommand;

desc_localizations!(captcha_disable_description);

impl CaptchaDisableCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let config = ctx.config(state).await?;
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

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
            .title(ctx.lang.captcha_disable_confirm_title())
            .description(
                ctx.lang
                    .captcha_disable_confirm_description(unverified, verification),
            )
            .build();

        let custom_id = CustomId::name("captcha-disable");
        let components = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(custom_id.to_string()),
                disabled: false,
                emoji: None,
                label: Some(ctx.lang.captcha_disable_confirm_button().to_string()),
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
#[command(
    name = "logs",
    desc = "Set the RaidProtect captcha logs channel",
    desc_localizations = "captcha_logs_description"
)]
pub struct CaptchaLogsCommand {
    /// Channel to send the logs to.
    #[command(channel_types = "guild_text")]
    channel: Id<ChannelMarker>,
}

desc_localizations!(captcha_logs_description);

impl CaptchaLogsCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let mut config = ctx.config(state).await?;
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

        // Ensure RaidProtect has permissions to send messages in the channel.
        let (permissions, _) = state
            .cache
            .permissions(ctx.guild_id)
            .await?
            .current_member()
            .await?
            .channel(self.channel)
            .await?;

        if !permissions.contains(Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS) {
            return Ok(embed::captcha::missing_logs_permission(ctx.lang));
        }

        // Update the config.
        config.captcha.logs = Some(self.channel);
        state.database.update_guild(&config).await?;

        // Send the embed.
        let embed = EmbedBuilder::new()
            .color(COLOR_GREEN)
            .title(ctx.lang.config_updated_title())
            .description(
                ctx.lang
                    .captcha_logs_confirm_description(self.channel.mention()),
            )
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-add",
    desc = "Add a role to the RaidProtect captcha autorole",
    desc_localizations = "captcha_autorole_add_description"
)]
pub struct CaptchaAutoroleAddCommand {
    /// Role to add to the autorole.
    role: Role,
}

desc_localizations!(captcha_autorole_add_description);

impl CaptchaAutoroleAddCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let mut config = ctx.config(state).await?;

        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

        // Ensure RaidProtect has permissions to give this role.
        let permissions = state
            .cache
            .permissions(ctx.guild_id)
            .await?
            .current_member()
            .await?;

        if !permissions.guild().contains(Permissions::MANAGE_ROLES) {
            return Ok(embed::captcha::missing_role_permission(ctx.lang));
        }

        if RoleOrdering::from(&self.role) >= permissions.highest_role() {
            return Ok(embed::captcha::role_hierarchy(ctx.lang));
        }

        // Update the configuration.
        if config.captcha.verified_roles.contains(&self.role.id) {
            return Ok(embed::captcha::role_already_added(ctx.lang));
        }

        if config.captcha.verified_roles.len() >= CaptchaConfig::MAX_VERIFIED_ROLES_LEN {
            return Ok(embed::captcha::role_too_many(ctx.lang));
        }

        config.captcha.verified_roles.push(self.role.id);
        state.database.update_guild(&config).await?;

        // Send the embed.
        let embed = EmbedBuilder::new()
            .color(COLOR_GREEN)
            .title(ctx.lang.config_updated_title())
            .description(
                ctx.lang
                    .captcha_autorole_add_confirm_description(self.role.mention()),
            )
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-remove",
    desc = "Remove a role from the RaidProtect captcha autorole",
    desc_localizations = "captcha_autorole_remove_description"
)]
pub struct CaptchaAutoroleRemoveCommand {
    /// Role to remove from the autorole.
    role: Id<RoleMarker>,
}

desc_localizations!(captcha_autorole_remove_description);

impl CaptchaAutoroleRemoveCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let mut config = ctx.config(state).await?;
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

        // Update the configuration.
        if !config.captcha.verified_roles.contains(&self.role) {
            return Ok(embed::captcha::role_not_configured(ctx.lang));
        }

        config.captcha.verified_roles.retain(|r| r != &self.role);
        state.database.update_guild(&config).await?;

        // Send the embed.
        let embed = EmbedBuilder::new()
            .color(COLOR_GREEN)
            .title(ctx.lang.config_updated_title())
            .description(
                ctx.lang
                    .captcha_autorole_remove_confirm_description(self.role.mention()),
            )
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-list",
    desc = "List the roles of the RaidProtect captcha autorole",
    desc_localizations = "captcha_autorole_list_description"
)]
pub struct CaptchaAutoroleListCommand;

desc_localizations!(captcha_autorole_list_description);

impl CaptchaAutoroleListCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let config = ctx.config(state).await?;
        if !config.captcha.enabled {
            return Ok(embed::captcha::not_enabled(ctx.lang));
        }

        // Get the roles list.
        let roles = config
            .captcha
            .verified_roles
            .iter()
            .map(|id| id.mention().to_string())
            .collect::<Vec<_>>()
            .join("\n");

        // Send the embed.
        let embed = if roles.is_empty() {
            EmbedBuilder::new()
                .color(COLOR_RED)
                .title(ctx.lang.captcha_autorole_empty_title())
                .description(ctx.lang.captcha_autorole_empty_description())
                .build()
        } else {
            EmbedBuilder::new()
                .color(COLOR_TRANSPARENT)
                .title(ctx.lang.captcha_autorole_list_title())
                .description(ctx.lang.captcha_autorole_list(roles))
                .build()
        };

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}
