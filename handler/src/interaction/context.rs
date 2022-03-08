//! Interaction context.
//!
//! This module contains types used to parse context from received interaction.

use thiserror::Error;
use twilight_model::{
    application::interaction::ApplicationCommand,
    id::{
        marker::{ChannelMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use crate::embed::error;

use super::callback::{InteractionError, InteractionErrorData};

/// Context of an [`ApplicationCommand`].
#[derive(Debug)]
pub enum CommandContext {
    /// Command received from a private message.
    Private(PrivateCommandContext),
    /// Command received from a guild.
    Guild(GuildCommandContext),
}

/// Context of an [`ApplicationCommand`] received from private messages.
#[derive(Debug)]
pub struct PrivateCommandContext {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// Token of the command.
    pub token: String,
    /// The channel the command was triggered from.
    pub channel: Id<ChannelMarker>,
    /// User that triggered the command.
    pub user: User,
    /// The user locale.
    pub locale: String,
}

impl TryFrom<ApplicationCommand> for PrivateCommandContext {
    type Error = CommandContextError;

    fn try_from(value: ApplicationCommand) -> Result<Self, Self::Error> {
        let user = value.user.ok_or(CommandContextError::MissingUser)?;

        Ok(PrivateCommandContext {
            id: value.id,
            token: value.token,
            channel: value.channel_id,
            user,
            locale: value.locale,
        })
    }
}

/// Context of an [`ApplicationCommand`] that is executed in a guild
#[derive(Debug)]
pub struct GuildCommandContext {
    /// ID of the interaction.
    id: Id<InteractionMarker>,
    // Context of the guild the command was triggered from.
    // guild: GuildContext,
    /// The channel the command was triggered from.
    channel_id: Id<ChannelMarker>,
}

/// Error occurred when initializing a [`CommandContext`].
#[derive(Debug, Error)]
pub enum CommandContextError {
    #[error("command must be executed in a guild")]
    GuildOnly,
    #[error("missing user data")]
    MissingUser,
}

impl InteractionError for CommandContextError {
    fn into_error(self) -> InteractionErrorData {
        match self {
            CommandContextError::GuildOnly => InteractionErrorData::callback(error::guild_only()),
            error => InteractionErrorData::internal(None, error),
        }
    }
}
