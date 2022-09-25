//! Captcha feature.

use std::time::Duration as StdDuration;

use raidprotect_model::cache::discord::CachedChannel;
use time::Duration;
use tracing::{error, trace};
use twilight_model::{
    application::component::{button::ButtonStyle, ActionRow, Button, Component},
    channel::Message,
    guild::Permissions,
    http::permission_overwrite::{
        PermissionOverwrite as HttpPermissionOverwrite,
        PermissionOverwriteType as HttpPermissionOverwriteType,
    },
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker},
        Id,
    },
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{
    cluster::ClusterState,
    interaction::{embed::COLOR_RED, util::CustomId},
    translations::Lang,
    util::TextProcessExt,
};

/// Default length of the generated captcha code.
pub const DEFAULT_LENGTH: usize = 5;

/// Default duration before the captcha expires.
pub const DEFAULT_DURATION: Duration = Duration::minutes(5);

/// Duration before the member is kicked for not completing the verification.
pub const KICK_AFTER: StdDuration = StdDuration::from_secs(10);

/// Maximum number of regenerations of the captcha code.
pub const MAX_RETRY: u8 = 2;

/// Send the captcha verification message in the verification channel.
///
/// This message is sent on when the captcha is enabled and automatically resent
/// if deleted.
pub async fn verification_message(
    channel: Id<ChannelMarker>,
    guild: Id<GuildMarker>,
    guild_lang: Lang,
    guild_name: &str,
    state: &ClusterState,
) -> Result<Message, anyhow::Error> {
    let embed = EmbedBuilder::new()
        .title(guild_lang.captcha_verification_title(guild_name.max_len(30)))
        .description(guild_lang.captcha_verification_description())
        .color(COLOR_RED)
        .build();

    let custom_id = CustomId::name("captcha-verify");
    let components = Component::ActionRow(ActionRow {
        components: vec![Component::Button(Button {
            custom_id: Some(custom_id.to_string()),
            disabled: false,
            emoji: None,
            label: Some(guild_lang.captcha_verification_button().to_owned()),
            style: ButtonStyle::Success,
            url: None,
        })],
    });

    let message = state
        .cache_http(guild)
        .create_message(channel)
        .await?
        .embeds(&[embed])?
        .components(&[components])?
        .exec()
        .await?
        .model()
        .await?;

    Ok(message)
}

/// Updates a channel permissions for the unverified role.
///
/// This will hide the channel from users having the unverified role.
/// Permissions will not be updated in the following cases:
/// - Channel is already hidden.
/// - Channel is private (hidden to the everyone role).
pub async fn update_channel_permissions(
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
        .http
        .update_channel_permission(channel.id, &permission_overwrite)
        .exec()
        .await
    {
        error!(error = ?error, "failed to update channel permissions for unverified role");
    }

    Ok(())
}
