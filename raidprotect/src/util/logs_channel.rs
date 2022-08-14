//! Get the logs channel of a guild.
//!
//! This module exports functions to get the logs channel of a specific guild.
//!
//! In case the channel is not configured for the current guild, a new one will
//! be automatically created. If a channel named `raidprotect-logs` is already
//! present, it will be reused.
//!
//! A simple locking mechanism is used to prevent multiple channels to be created
//! at the same time.

use std::collections::HashMap;

use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use raidprotect_model::cache::model::CachedChannel;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, trace, warn};
use twilight_model::{
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
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};
use twilight_util::builder::embed::EmbedBuilder;

use crate::{cluster::ClusterState, interaction::embed::COLOR_RED, translations::Lang};

/// Default logs channel name.
const DEFAULT_LOGS_NAME: &str = "raidprotect-logs";

type PendingChannelsMap = HashMap<Id<GuildMarker>, broadcast::Sender<Id<ChannelMarker>>>;

/// Logs channel creation queue.
///
/// This hold a list of pending logs channels being created. A [`broadcast::Sender`]
/// is hold to notify when the channel has been created.
static PENDING_CHANNELS: Lazy<RwLock<PendingChannelsMap>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Get the logs channel of a guild.
///
/// The `lang` argument should be the guild language, not the user language.
///
/// See the [module documentation](super) for more information.
pub async fn guild_logs_channel(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    logs_channel: Option<Id<ChannelMarker>>,
    lang: Lang,
) -> Result<Id<ChannelMarker>, anyhow::Error> {
    // If a channel is already configured, ensure it exists and return it.
    if let Some(channel) = logs_channel {
        let cached = state.redis().get::<CachedChannel>(&channel).await?;

        if cached.is_some() {
            return Ok(channel);
        }
    }

    // To avoid creating multiple channels, we use the `PENDING_CHANNELS` map to
    // store a lock for each guild. The lock is a broadcast channel, so we can
    // send the created channel to all the pending tasks.
    let receiver = {
        let pending_channels = PENDING_CHANNELS.read().await;
        let sender = pending_channels.get(&guild);

        sender.map(|s| s.subscribe())
    };

    // If a channel is being created, wait and return it's id
    if let Some(mut rx) = receiver {
        trace!(guild = ?guild, "waiting for logs channel to be created");

        match rx.recv().await {
            Ok(channel) => return Ok(channel),
            Err(_) => return Err(anyhow!("error while waiting for logs channel creation")),
        }
    }

    // Create a new logs channel
    trace!(guild = ?guild, "creating logs channel");

    configure_logs_channel(state, guild, lang).await
}

/// Try to find an existing logs channel, or create a new one.
async fn configure_logs_channel(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    lang: Lang,
) -> Result<Id<ChannelMarker>, anyhow::Error> {
    // Add a lock to the pending channels map.
    let sender = {
        let mut pending_channels = PENDING_CHANNELS.write().await;
        let (sender, _) = broadcast::channel(1);

        pending_channels.insert(guild, sender.clone());

        sender
    };

    // Try to find an existing channel .
    let guild_channels = state.redis().guild_channels(guild).await?;
    let logs_channel = guild_channels.iter().find(|channel| match channel {
        CachedChannel::Text(channel) => channel.name == DEFAULT_LOGS_NAME,
        _ => false,
    });

    let logs_channel = match logs_channel {
        Some(channel) => update_logs_permissions(state, channel, guild).await,
        None => create_logs_channel(state, guild, lang).await?,
    };

    // Update the guild configuration
    let mut config = state.mongodb().get_guild_or_create(guild).await?;
    config.logs_chan = Some(logs_channel);
    state.mongodb().update_guild(&config).await?;

    // Notify pending tasks that the channel has been created.
    sender.send(logs_channel).ok();

    Ok(logs_channel)
}

/// Update permissions for an existing logs channel.
async fn update_logs_permissions(
    state: &ClusterState,
    channel: &CachedChannel,
    guild: Id<GuildMarker>,
) -> Id<ChannelMarker> {
    let permission_overwrite = HttpPermissionOverwrite {
        id: state.current_user().cast(),
        kind: HttpPermissionOverwriteType::Member,
        allow: Some(
            Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS,
        ),
        deny: None,
    };

    if let Err(error) = state
        .cache_http(guild)
        .update_channel_permission(channel.id(), &permission_overwrite)
        .await
    {
        warn!(error = ?error, guild = ?guild, "failed to update existing logs channel permissions");
    }

    channel.id()
}

/// Create a new logs channel in the guild.
async fn create_logs_channel(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    lang: Lang,
) -> Result<Id<ChannelMarker>, anyhow::Error> {
    // Hide the channel to the everyone role.
    // Only users with the `ADMINISTRATOR` permission will be able to see it.
    let permission_overwrite = [
        PermissionOverwrite {
            id: guild.cast(),
            kind: PermissionOverwriteType::Role,
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
        },
        PermissionOverwrite {
            id: state.current_user().cast(),
            kind: PermissionOverwriteType::Member,
            allow: Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::EMBED_LINKS,
            deny: Permissions::empty(),
        },
    ];

    let channel = match state
        .cache_http(guild)
        .create_guild_channel(DEFAULT_LOGS_NAME)
        .await?
        .kind(ChannelType::GuildText)
        .permission_overwrites(&permission_overwrite)
        .exec()
        .await
    {
        Ok(response) => response.model().await?,
        Err(error) => {
            error!(error = ?error, guild = ?guild, "failed to create logs channel");

            return Err(error).context("failed to create logs channel");
        }
    };

    // Send an initial message to the channel.
    if let Err(error) = send_logs_message(state, guild, channel.id, lang).await {
        warn!(error = ?error, guild = ?guild, "error while sending initial logs channel message");
    }

    Ok(channel.id)
}

/// Send an informational message to the logs channel.
async fn send_logs_message(
    state: &ClusterState,
    guild: Id<GuildMarker>,
    channel: Id<ChannelMarker>,
    lang: Lang,
) -> Result<(), anyhow::Error> {
    let embed = EmbedBuilder::new()
        .title(lang.logs_creation_title())
        .color(COLOR_RED)
        .description(lang.logs_creation_description())
        .build();

    state
        .cache_http(guild)
        .create_message(channel)
        .await?
        .embeds(&[embed])?
        .exec()
        .await?;

    Ok(())
}
