//! Interaction context.
//!
//! This module contains types used to parse context from received interaction.

use raidprotect_model::{collection, mongodb::MongoDbError, ClusterState};
use thiserror::Error;
use twilight_http::client::InteractionClient;
use twilight_model::{
    application::interaction::{application_command::CommandData, ApplicationCommand},
    guild::PartialMember,
    id::{
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use super::response::{InteractionError, InteractionErrorKind};

/// Context of an [`ApplicationCommand`].
#[derive(Debug)]

pub struct CommandContext {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// ID of the associated application.
    pub application_id: Id<ApplicationMarker>,
    /// Token of the command.
    pub token: String,
    /// Data from the invoked command.
    pub data: CommandData,
    /// The channel the command was triggered from.
    pub channel: Id<ChannelMarker>,
    /// If command occurred in a guild, the guild configuration from database.
    pub guild: Option<collection::Guild>,
    /// User that triggered the command.
    pub user: User,
    /// If command occurred in a guild, the member that triggered the command.
    pub member: Option<PartialMember>,
    /// The user locale.
    pub locale: String,
}

impl CommandContext {
    /// Initialize a new [`CommandContext`] from an incoming command data
    pub async fn from_command(
        command: ApplicationCommand,
        state: &ClusterState,
    ) -> Result<Self, CommandContextError> {
        match command.guild_id {
            Some(guild_id) => Self::from_guild_command(command, state, guild_id).await,
            None => Self::from_private_command(command),
        }
    }

    /// Initialize context from a command that occurred in a guild.
    async fn from_guild_command(
        command: ApplicationCommand,
        state: &ClusterState,
        guild_id: Id<GuildMarker>,
    ) -> Result<Self, CommandContextError> {
        let member = command.member.ok_or(CommandContextError::MissingMember)?;
        let user = member
            .user
            .clone()
            .ok_or(CommandContextError::MissingUser)?;

        let guild = state.mongodb().get_guild_or_create(guild_id).await?;

        Ok(Self {
            id: command.id,
            application_id: command.application_id,
            token: command.token,
            data: command.data,
            channel: command.channel_id,
            guild: Some(guild),
            user,
            member: Some(member),
            locale: command.locale,
        })
    }

    /// Initialize context from a command that occurred in private messages.
    fn from_private_command(command: ApplicationCommand) -> Result<Self, CommandContextError> {
        let user = command.user.ok_or(CommandContextError::MissingUser)?;

        Ok(Self {
            id: command.id,
            application_id: command.application_id,
            token: command.token,
            data: command.data,
            channel: command.channel_id,
            guild: None,
            user,
            member: None,
            locale: command.locale,
        })
    }

    /// Get the [`InteractionClient`] associated with the current context.
    pub fn interaction<'state>(&self, state: &'state ClusterState) -> InteractionClient<'state> {
        state.http().interaction(self.application_id)
    }
}

/// Error occurred when initializing a [`CommandContext`].
#[derive(Debug, Error)]
pub enum CommandContextError {
    #[error("missing user data")]
    MissingUser,
    #[error("missing member data")]
    MissingMember,
    #[error(transparent)]
    MongoDb(#[from] MongoDbError),
}

impl InteractionError for CommandContextError {
    const INTERACTION_NAME: &'static str = "context";

    fn into_error(self) -> InteractionErrorKind {
        InteractionErrorKind::internal(self)
    }
}
