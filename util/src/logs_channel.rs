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

use lazy_static::lazy_static;
use raidprotect_cache::{http::CacheHttpError, model::CachedChannel, redis::RedisClientError};
use raidprotect_model::mongodb::MongoDbError;
use raidprotect_state::ClusterState;
use raidprotect_translations::Lang;
use thiserror::Error;
use tokio::sync::{broadcast, RwLock};
use twilight_http::{error::Error as HttpError, response::DeserializeBodyError};
use twilight_model::{
    channel::{
        permission_overwrite::{PermissionOverwrite, PermissionOverwriteType},
        ChannelType,
    },
    guild::Permissions,
    id::{
        marker::{ChannelMarker, GuildMarker},
        Id,
    },
};
use twilight_util::builder::embed::EmbedBuilder;
use twilight_validate::message::MessageValidationError;

use crate::COLOR_RED;

/// Default logs channel name.
const DEFAULT_LOGS_NAME: &str = "raidprotect-logs";

lazy_static! {
    /// Logs channel creation queue.
    ///
    /// This hold a list of pending logs channels being created. A [`broadcast::Sender`]
    /// is hold to notify when the channel has been created.
    static ref PENDING_CHANNELS: RwLock<HashMap<Id<GuildMarker>, broadcast::Sender<Id<ChannelMarker>>>> =
        RwLock::new(HashMap::new());
}

/// Get the logs channel of a guild.
///
/// See the [module documentation](super) for more information.
pub async fn guild_logs_channel(
    guild_id: Id<GuildMarker>,
    logs_chan: Option<Id<ChannelMarker>>,
    state: &ClusterState,
) -> Result<Id<ChannelMarker>, LogsChannelError> {
    // If a channel is given, ensure the channel exists
    if let Some(logs_chan) = logs_chan {
        if state
            .redis()
            .get::<CachedChannel>(&logs_chan)
            .await?
            .is_some()
        {
            return Ok(logs_chan);
        }
    }

    // Create the logs channel
    if let Some(tx) = PENDING_CHANNELS.read().await.get(&guild_id) {
        let mut rx = tx.subscribe();

        match rx.recv().await {
            Ok(logs_chan) => Ok(logs_chan),
            Err(_) => Err(LogsChannelError::PendingFailed),
        }
    } else {
        create_logs_channel(guild_id, state).await
    }
}

/// Create a new logs channel
async fn create_logs_channel(
    guild_id: Id<GuildMarker>,
    state: &ClusterState,
) -> Result<Id<ChannelMarker>, LogsChannelError> {
    let (tx, _) = broadcast::channel(1);
    PENDING_CHANNELS.write().await.insert(guild_id, tx.clone());

    // Check if a channel named `raidprotect-logs` already exists.
    // If not, create a new channel.
    let channels = state.redis().guild_channels(guild_id).await?;
    let logs_channel = channels.iter().find(|channel| match channel {
        CachedChannel::Text(channel) => channel.name == DEFAULT_LOGS_NAME,
        _ => false,
    });

    // Create channel if not exists
    let logs_channel_id = if let Some(channel) = logs_channel {
        channel.id()
    } else {
        // Deny everyone role to see the channel
        let permission_overwrite = PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            id: guild_id.cast(),
            kind: PermissionOverwriteType::Role,
        };

        let channel = state
            .cache_http(guild_id)
            .create_guild_channel(DEFAULT_LOGS_NAME)
            .await?
            .kind(ChannelType::GuildText)
            .permission_overwrites(&[permission_overwrite])
            .exec()
            .await?
            .model()
            .await?;

        channel.id
    };

    // Update channel in configuration
    let mut guild = state.mongodb().get_guild_or_create(guild_id).await?;
    guild.config.logs_chan = Some(logs_channel_id);
    state.mongodb().update_guild(&guild).await?;

    tx.send(logs_channel_id).ok();

    // Send message in channel
    let embed = EmbedBuilder::new()
        .title(Lang::Fr.logs_creation_title())
        .color(COLOR_RED)
        .description(Lang::Fr.logs_creation_description())
        .build();

    state
        .cache_http(guild_id)
        .create_message(logs_channel_id)
        .await?
        .embeds(&[embed])?
        .exec()
        .await
        .ok(); // Do not fail if message cannot be sent

    Ok(logs_channel_id)
}

#[derive(Debug, Error)]
pub enum LogsChannelError {
    #[error("redis error: {0}")]
    Redis(#[from] RedisClientError),
    #[error("mongodb error: {0}")]
    MongoDb(#[from] MongoDbError),
    #[error("http error: {0}")]
    CacheHttp(#[from] CacheHttpError),
    #[error("http error: {0}")]
    HttpError(#[from] HttpError),
    #[error("http error: {0}")]
    DeserializeBody(#[from] DeserializeBodyError),
    #[error("message validation: {0}")]
    MessageValidationError(#[from] MessageValidationError),
    #[error("pending channel creation failed")]
    PendingFailed,
}
