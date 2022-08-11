//! Captcha button components.

use anyhow::Context;
use raidprotect_model::cache::model::CachedGuild;
use tracing::error;
use twilight_http::request::AuditLogReason;
use twilight_mention::Mention;
use twilight_model::{
    application::{
        component::{button::ButtonStyle, ActionRow, Button, Component},
        interaction::Interaction,
    },
    channel::{
        permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
        ChannelType,
    },
    guild::Permissions,
};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::{
    cluster::ClusterState,
    interaction::{
        embed::{self, COLOR_GREEN, COLOR_RED},
        response::InteractionResponse,
        util::{CustomId, InteractionExt},
    },
    translations::Lang,
    util::TextProcessExt,
};

/// Captcha enabling button.
///
/// This type handle the button used to enable the captcha (sent by the
/// `/config captcha enable` command).
///
/// During the captcha activation, the following actions are performed:
/// - A `#verification` channel and a `@Unverified` role are created.
/// - A message is sent in the `#verification` channel, with a button to
///   verify the user.
/// - Each channel in the guild is configured to be hidden to the `@Unverified`
///   role.
pub struct CaptchaEnable;

impl CaptchaEnable {
    pub async fn handle(
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let guild = interaction.guild()?;
        let cached_guild = state
            .redis()
            .get::<CachedGuild>(&guild.id)
            .await?
            .context("missing cached guild")?;
        let mut config = state.mongodb().get_guild_or_create(guild.id).await?;

        let lang = interaction.locale()?;
        let guild_lang = Lang::from(&*config.lang);

        // Ensure the bot has the required permissions to enable the captcha.
        //
        // The permissions of the user performing the action are not checked,
        // since the button is sent attached to an ephemeral message.
        let permissions = state
            .redis()
            .permissions(guild.id)
            .await?
            .current_member()
            .await?;

        // The bot needs the `MANAGE_CHANNELS` and `MANAGE_ROLES` permissions.
        if !permissions
            .guild()
            .contains(Permissions::MANAGE_CHANNELS | Permissions::MANAGE_ROLES)
        {
            return Ok(embed::captcha::missing_enable_permission(lang));
        }

        // Create the `#verification` channel and the `@Unverified` role.
        let unverified_role = match state
            .http()
            .create_role(guild.id)
            .name(guild_lang.captcha_role_name())
            .color(0x99AAB5) // Default grey color
            .permissions(Permissions::empty())
            .reason(guild_lang.captcha_enable_reason())?
            .exec()
            .await
        {
            Ok(response) => response.model().await?,
            Err(err) => {
                error!(error = ?err, "failed to create the unverified role");

                return Ok(embed::captcha::role_error(lang));
            }
        };

        let channel_permissions = vec![
            // Hide channel for @everyone.
            PermissionOverwrite {
                id: guild.id.cast(),
                kind: PermissionOverwriteType::Role,
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
            },
            // Show the channel for @Unverified.
            PermissionOverwrite {
                id: unverified_role.id.cast(),
                kind: PermissionOverwriteType::Role,
                allow: Permissions::VIEW_CHANNEL | Permissions::READ_MESSAGE_HISTORY,
                deny: Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS,
            },
            // Ensure the bot has necessary permissions in the channel.
            PermissionOverwrite {
                id: state.current_user().cast(),
                kind: PermissionOverwriteType::Member,
                allow: Permissions::VIEW_CHANNEL
                    | Permissions::SEND_MESSAGES
                    | Permissions::EMBED_LINKS,
                deny: Permissions::empty(),
            },
        ];

        let verification_channel = match state
            .http()
            .create_guild_channel(guild.id, guild_lang.captcha_channel_name())?
            .kind(ChannelType::GuildText)
            .position(0) // Put the channel at the top of the list.
            .permission_overwrites(&channel_permissions)
            .reason(guild_lang.captcha_enable_reason())?
            .exec()
            .await
        {
            Ok(response) => response.model().await?,
            Err(err) => {
                error!(error = ?err, "failed to create the verification channel");

                return Ok(embed::captcha::channel_error(lang));
            }
        };

        // Send the verification message in the channel.
        let embed = EmbedBuilder::new()
            .title(guild_lang.captcha_verification_title(cached_guild.name.truncate(30)))
            .description(guild_lang.captcha_verification_description())
            .color(COLOR_RED)
            .build();

        let custom_id = CustomId::name("captcha-verify");
        let components = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: Some(custom_id.to_string()),
                disabled: false,
                emoji: None,
                label: Some(lang.captcha_verification_button().to_string()),
                style: ButtonStyle::Success,
                url: None,
            })],
        });

        let message = match state
            .http()
            .create_message(verification_channel.id)
            .embeds(&[embed])?
            .components(&[components])?
            .exec()
            .await
        {
            Ok(response) => response.model().await?,
            Err(err) => {
                error!(error = ?err, "failed to send the verification message");

                return Ok(embed::error::internal_error(lang));
            }
        };

        // Update the guild configuration.
        config.captcha.enabled = true;
        config.captcha.channel = Some(verification_channel.id);
        config.captcha.message = Some(message.id);
        config.captcha.role = Some(unverified_role.id);

        state.mongodb().update_guild(&config).await?;

        // Start the configuration of channels permissions.

        // Send the confirmation message.
        let embed = EmbedBuilder::new()
            .title(lang.captcha_enabled_title())
            .color(COLOR_GREEN)
            .description(lang.captcha_enabled_description(
                verification_channel.mention(),
                unverified_role.mention(),
            ))
            .field(EmbedFieldBuilder::new(
                lang.captcha_enabled_roles_title(),
                lang.captcha_enabled_roles_description(),
            ))
            .field(EmbedFieldBuilder::new(
                lang.captcha_enabled_rename_title(),
                lang.captcha_enabled_rename_description(
                    verification_channel.mention(),
                    unverified_role.mention(),
                ),
            ))
            .build();

        Ok(InteractionResponse::EphemeralEmbed(embed))
    }
}
