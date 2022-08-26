//! Captcha enable button.

use std::{sync::Arc, time::Duration};

use anyhow::Context;
use raidprotect_model::cache::model::{CachedChannel, CachedGuild};
use tracing::{debug, error, trace};
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
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
    },
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
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
    util::{guild_logs_channel, TextProcessExt},
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
        state: Arc<ClusterState>,
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
        let author_id = interaction.author_id().context("missing author id")?;

        // Ensure the captcha is not already enabled.
        // The button could be clicked twice, this is a safety check.
        if config.captcha.enabled {
            return Ok(embed::captcha::already_enabled(lang));
        }

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
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(error) = configure_channels(
                state_clone,
                guild.id,
                unverified_role.id,
                verification_channel.id,
            )
            .await
            {
                error!(error = ?error, guild = ?guild.id, "failed to configure captcha channels permissions");
            }
        });

        // Send message in logs channel.
        tokio::spawn(async move {
            if let Err(error) =
                logs_message(state, guild.id, config.logs_chan, author_id, guild_lang).await
            {
                error!(error = ?error, guild = ?guild.id, "failed to send captcha enable logs message");
            }
        });

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

/// Send a message in the logs channel to notify that the captcha has been
/// enabled.
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
        .description(lang.captcha_enabled_log(user.mention()))
        .build();

    state
        .http()
        .create_message(channel)
        .embeds(&[embed])?
        .exec()
        .await?;

    Ok(())
}

/// Configure the permissions of the guild channels.
///
/// For the unverified role to work, all the channels of the guild must be
/// hidden to the role. This function should be used as a background task.
///
/// The function will iterate over all guilds channels and compute permissions
/// for the unverified role. If the role can see the channel, the permissions
/// will be updated accordingly (see [`update_channel_permissions`]).
///
/// The category channels are iterated first, since a lot of channels can inherit
/// from their permissions. The verification channel is skipped.
async fn configure_channels(
    state: Arc<ClusterState>,
    guild: Id<GuildMarker>,
    role: Id<RoleMarker>,
    verification: Id<ChannelMarker>,
) -> Result<(), anyhow::Error> {
    let guild_channels = state.redis().guild_channels(guild).await?;

    let mut categories = Vec::new();
    let mut channels = Vec::new();

    for channel in guild_channels {
        // Permissions are not updated for the verification channel.
        if channel.id == verification {
            continue;
        }

        // Threads inherit permissions from their parent channel.
        if channel.is_thread() {
            continue;
        }

        if channel.kind == ChannelType::GuildCategory {
            categories.push(channel);
        } else {
            channels.push(channel.id);
        }
    }

    // Update permissions for the category channels first.
    // This will reduce the number of requests to the API since a lot of channels
    // can inherit from their category.
    for channel in categories {
        update_channel_permissions(&state, &channel, guild, role).await?;
    }

    // Small delay to ensure the cache is updated with the new permissions.
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Update permissions for the remaining channels.
    // Since the permissions for these channels could have been updated by the
    // category channels, the channel is retrieved from the cache again.
    for channel in channels {
        let channel = match state.redis().get::<CachedChannel>(&channel).await? {
            Some(channel) => channel,
            None => {
                // Since some delay could have been elapsed since the previous
                // cache request, the channel could have been deleted.
                debug!(channel = ?channel, guild = ?guild, "channel no longer in cache during captcha configuration");

                continue;
            }
        };

        update_channel_permissions(&state, &channel, guild, role).await?;
    }

    Ok(())
}

/// Updates a channel permissions for the unverified role.
async fn update_channel_permissions(
    state: &ClusterState,
    channel: &CachedChannel,
    guild: Id<GuildMarker>,
    role: Id<RoleMarker>,
) -> Result<(), anyhow::Error> {
    trace!(channel = ?channel.id, role = ?role, guild = ?guild, "updating channel permissions for captcha");

    // Get permissions for the unverified role. The permissions for everyone
    // are also retrieved to avoid updating permissions unnecessarily for private
    // channels.
    let permissions = channel.permission_overwrites.clone().unwrap_or_default();

    let role_permissions = permissions.iter().find(|p| p.id == role.cast());
    let everyone_permissions = permissions.iter().find(|p| p.id == guild.cast());

    // Skip updating permissions if the channel is private.
    if everyone_permissions
        .map(|p| p.deny)
        .unwrap_or(Permissions::empty())
        .contains(Permissions::VIEW_CHANNEL)
    {
        return Ok(());
    }

    // Skip updating permissions if the role is already denied to view the channel.
    // This will be the case if the channel is a text channel that inherits from
    // a category channel (that should have been updated first).
    if role_permissions
        .map(|p| p.deny)
        .unwrap_or(Permissions::empty())
        .contains(Permissions::VIEW_CHANNEL)
    {
        return Ok(());
    }

    // Update the permissions for the unverified role.
    let permission_overwrite = HttpPermissionOverwrite {
        id: role.cast(),
        kind: HttpPermissionOverwriteType::Role,
        allow: None,
        deny: Some(Permissions::VIEW_CHANNEL),
    };

    if let Err(error) = state
        .http()
        .update_channel_permission(channel.id, &permission_overwrite)
        .exec()
        .await
    {
        error!(error = ?error, "failed to update channel permissions for unverified role");
    }

    Ok(())
}
