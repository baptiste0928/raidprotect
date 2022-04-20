//! Interaction context.
//!
//! This module contains types used to parse context from received interaction.

use raidprotect_model::{collection, mongodb::MongoDbError, ClusterState};
use thiserror::Error;
use twilight_http::client::InteractionClient;
use twilight_model::{
    application::interaction::{
        application_command::CommandData, message_component::MessageComponentInteractionData,
        ApplicationCommand, MessageComponentInteraction,
    },
    guild::PartialMember,
    id::{
        marker::{ApplicationMarker, ChannelMarker, GuildMarker, InteractionMarker},
        Id,
    },
    user::User,
};

use super::response::{InteractionError, InteractionErrorKind};

/// Context of an [`ApplicationCommand`] or [`MessageComponentInteraction`].
///
/// This type is used for both command and interaction components as the types
/// are similar. A generic parameter is used for the `data` field.
#[derive(Debug)]
pub struct InteractionContext<D> {
    /// ID of the command.
    pub id: Id<InteractionMarker>,
    /// ID of the associated application.
    pub application_id: Id<ApplicationMarker>,
    /// Token of the command.
    pub token: String,
    /// Data from the invoked command.
    pub data: D,
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

impl<D> InteractionContext<D> {
    /// Get the [`InteractionClient`] associated with the current context.
    pub fn interaction<'state>(&self, state: &'state ClusterState) -> InteractionClient<'state> {
        state.http().interaction(self.application_id)
    }
}

impl InteractionContext<CommandData> {
    /// Initialize a new [`InteractionContext`] from an [`ApplicationCommand`].
    pub async fn from_command(
        command: ApplicationCommand,
        state: &ClusterState,
    ) -> Result<Self, InteractionContextError> {
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
    ) -> Result<Self, InteractionContextError> {
        let member = command
            .member
            .ok_or(InteractionContextError::MissingMember)?;
        let user = member
            .user
            .clone()
            .ok_or(InteractionContextError::MissingUser)?;

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
    fn from_private_command(command: ApplicationCommand) -> Result<Self, InteractionContextError> {
        let user = command.user.ok_or(InteractionContextError::MissingUser)?;

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
}

impl InteractionContext<MessageComponentInteractionData> {
    /// Initialize a new [`InteractionContext`] from an [`MessageComponentInteraction`].
    pub async fn from_component(
        component: MessageComponentInteraction,
        state: &ClusterState,
    ) -> Result<Self, InteractionContextError> {
        match component.guild_id {
            Some(guild_id) => Self::from_guild_component(component, state, guild_id).await,
            None => Self::from_private_component(component),
        }
    }

    /// Initialize context from a component triggered in a guild.
    ///
    /// The implementation is similar to `from_guild_command`.
    async fn from_guild_component(
        component: MessageComponentInteraction,
        state: &ClusterState,
        guild_id: Id<GuildMarker>,
    ) -> Result<Self, InteractionContextError> {
        let member = component
            .member
            .ok_or(InteractionContextError::MissingMember)?;
        let user = member
            .user
            .clone()
            .ok_or(InteractionContextError::MissingUser)?;

        let guild = state.mongodb().get_guild_or_create(guild_id).await?;

        Ok(Self {
            id: component.id,
            application_id: component.application_id,
            token: component.token,
            data: component.data,
            channel: component.channel_id,
            guild: Some(guild),
            user,
            member: Some(member),
            locale: component.locale,
        })
    }

    /// Initialize context from a component triggered in private messages.
    fn from_private_component(
        component: MessageComponentInteraction,
    ) -> Result<Self, InteractionContextError> {
        let user = component.user.ok_or(InteractionContextError::MissingUser)?;

        Ok(Self {
            id: component.id,
            application_id: component.application_id,
            token: component.token,
            data: component.data,
            channel: component.channel_id,
            guild: None,
            user,
            member: None,
            locale: component.locale,
        })
    }
}

/// Error occurred when initializing a [`InteractionContext`].
#[derive(Debug, Error)]
pub enum InteractionContextError {
    #[error("missing user data")]
    MissingUser,
    #[error("missing member data")]
    MissingMember,
    #[error(transparent)]
    MongoDb(#[from] MongoDbError),
}

impl InteractionError for InteractionContextError {
    const INTERACTION_NAME: &'static str = "context";

    fn into_error(self) -> InteractionErrorKind {
        InteractionErrorKind::internal(self)
    }
}
