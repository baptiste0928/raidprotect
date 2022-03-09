//! Interaction context.
//!
//! This module contains types used to parse context from received interaction.

use raidprotect_gateway::event::context::{BaseContext, ContextError, EventContext, GuildContext};
use thiserror::Error;
use twilight_http::Error as HttpError;
use twilight_model::{
    application::{callback::InteractionResponse, interaction::ApplicationCommand},
    guild::PartialMember,
    id::{
        marker::{ChannelMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use crate::embed::error;

use super::callback::{InteractionError, InteractionErrorData};

/// Data used to respond to an incoming command.
#[derive(Debug)]
pub struct CommandCallback {
    /// ID of the command.
    id: Id<InteractionMarker>,
    /// Token of the command.
    token: String,
}

impl CommandCallback {
    /// Initialize a new [`CommandCallback`] from an incoming command data.
    pub fn from_command(command: &ApplicationCommand) -> Self {
        Self {
            id: command.id,
            token: command.token.clone(),
        }
    }

    /// Respond to an interaction
    pub async fn exec(
        &self,
        context: &impl EventContext,
        response: &InteractionResponse,
    ) -> Result<(), HttpError> {
        context
            .interaction()
            .interaction_callback(self.id, &self.token, response)
            .exec()
            .await?;

        Ok(())
    }
}

/// Context of an [`ApplicationCommand`].
#[derive(Debug)]
pub enum CommandContext {
    /// Command received from a private message.
    Private(PrivateCommandContext),
    /// Command received from a guild.
    Guild(GuildCommandContext),
}

impl CommandContext {
    /// Initialize a new [`CommandContext`] from an incoming command data.
    pub fn from_command(
        command: ApplicationCommand,
        ctx: BaseContext,
    ) -> Result<Self, CommandContextError> {
        if command.guild_id.is_some() {
            Ok(Self::Guild(GuildCommandContext::from_command(
                command, ctx,
            )?))
        } else {
            Ok(Self::Private(PrivateCommandContext::from_command(command)?))
        }
    }

    /// Get the command [`Id`] and token.
    pub fn callback(&self) -> (Id<InteractionMarker>, &str) {
        match self {
            CommandContext::Private(ctx) => (ctx.id, &ctx.token),
            CommandContext::Guild(ctx) => (ctx.id, &ctx.token),
        }
    }

    /// Get the channel the command was triggered from.
    pub fn channel(&self) -> Id<ChannelMarker> {
        match self {
            CommandContext::Private(ctx) => ctx.channel,
            CommandContext::Guild(ctx) => ctx.channel,
        }
    }

    /// Get the user that triggered the command.
    pub fn user(&self) -> &User {
        match self {
            CommandContext::Private(ctx) => &ctx.user,
            CommandContext::Guild(ctx) => &ctx.user,
        }
    }

    /// Get the user locale.
    pub fn locale(&self) -> &str {
        match self {
            CommandContext::Private(ctx) => &ctx.locale,
            CommandContext::Guild(ctx) => &ctx.locale,
        }
    }
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

impl PrivateCommandContext {
    /// Initialize a new [`PrivateCommandContext`] from an incoming command data.
    pub fn from_command(command: ApplicationCommand) -> Result<Self, CommandContextError> {
        let user = command.user.ok_or(CommandContextError::MissingUser)?;

        Ok(PrivateCommandContext {
            id: command.id,
            token: command.token,
            channel: command.channel_id,
            user,
            locale: command.locale,
        })
    }
}

/// Context of an [`ApplicationCommand`] that is executed in a guild
#[derive(Debug)]
pub struct GuildCommandContext {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// Token of the command.
    pub token: String,
    // Context of the guild the command was triggered from.
    pub guild: GuildContext,
    /// The channel the command was triggered from.
    pub channel: Id<ChannelMarker>,
    /// User that triggered the command.
    ///
    /// This value is cloned from the user contained in [`PartialMember`].
    pub user: User,
    /// Member that triggered the command.
    pub member: PartialMember,
    /// The user locale.
    pub locale: String,
}

impl GuildCommandContext {
    /// Initialize a new [`GuildCommandContext`] from an incoming command data.
    pub fn from_command(
        command: ApplicationCommand,
        ctx: BaseContext,
    ) -> Result<Self, CommandContextError> {
        let guild_id = command.guild_id.ok_or(CommandContextError::GuildOnly)?;
        let member = command.member.ok_or(CommandContextError::MissingMember)?;
        let user = member
            .user
            .clone()
            .ok_or(CommandContextError::MissingUser)?;

        let guild = GuildContext::from_context(ctx, guild_id)?;

        Ok(GuildCommandContext {
            id: command.id,
            token: command.token,
            channel: command.channel_id,
            user,
            member,
            guild,
            locale: command.locale,
        })
    }
}

/// Error occurred when initializing a [`CommandContext`].
#[derive(Debug, Error)]
pub enum CommandContextError {
    #[error("command must be executed in a guild")]
    GuildOnly,
    #[error("missing user data")]
    MissingUser,
    #[error("missing member data")]
    MissingMember,
    #[error("failed to intialize guild context: {0}")]
    Context(#[from] ContextError),
}

impl InteractionError for CommandContextError {
    fn into_error(self) -> InteractionErrorData {
        match self {
            CommandContextError::GuildOnly => InteractionErrorData::callback(error::guild_only()),
            error => InteractionErrorData::internal(None, error),
        }
    }
}
