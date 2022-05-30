//! Profile command.
//!
//! This command shows basic information about a given user.

use std::time::Duration;

use raidprotect_model::cache::RedisClientError;
use thiserror::Error;
use tracing::instrument;
use twilight_interactions::{
    command::{CommandModel, CreateCommand, ResolvedUser},
    error::ParseError,
};
use twilight_mention::{
    timestamp::{Timestamp, TimestampStyle},
    Mention,
};
use twilight_model::application::{
    component::{button::ButtonStyle, ActionRow, Button, Component},
    interaction::application_command::CommandData,
};
use twilight_util::{
    builder::{
        embed::{
            image_source::ImageSourceUrlError, EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder,
            ImageSource,
        },
        InteractionResponseDataBuilder,
    },
    snowflake::Snowflake,
};
use twilight_validate::embed::EmbedValidationError;

use crate::{
    cluster::ClusterState,
    interaction::{
        component::post_in_chat::PostInChat,
        context::InteractionContext,
        embed::COLOR_TRANSPARENT,
        response::{InteractionError, InteractionErrorKind},
    },
    translations::Lang,
    util::resource::avatar_url,
};

/// Profile command model.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "profile",
    desc = "Show information about a user profile",
    dm_permission = true
)]
pub struct ProfileCommand {
    /// Mention or ID of the user.
    pub user: ResolvedUser,
}

impl ProfileCommand {
    /// Handle interaction for this command.
    #[instrument]
    pub async fn handle(
        context: InteractionContext<CommandData>,
        state: &ClusterState,
    ) -> Result<PostInChat, ProfileCommandError> {
        let parsed = ProfileCommand::from_interaction(context.data.into())?;
        let user = parsed.user.resolved;

        let avatar = avatar_url(&user, "jpeg", 1024);
        let mut embed = EmbedBuilder::new()
            .color(COLOR_TRANSPARENT)
            .title(Lang::Fr.profile_title(user.discriminator(), &user.name))
            .footer(EmbedFooterBuilder::new(format!("ID: {}", user.id)).build())
            .thumbnail(ImageSource::url(&avatar)?);

        // User profile creation time.
        let created_at = Duration::from_millis(user.id.timestamp() as u64).as_secs();
        let timestamp_long = Timestamp::new(created_at, Some(TimestampStyle::LongDate)).mention();
        let timestamp_relative =
            Timestamp::new(created_at, Some(TimestampStyle::RelativeTime)).mention();

        embed = embed.field(EmbedFieldBuilder::new(
            Lang::Fr.profile_created_at(),
            format!("{timestamp_long} ({timestamp_relative})"),
        ));

        // Member join date.
        if let Some(member) = parsed.user.member {
            let joined_at = member.joined_at.as_secs();
            let timestamp_long =
                Timestamp::new(joined_at as u64, Some(TimestampStyle::LongDate)).mention();
            let timestamp_relative =
                Timestamp::new(joined_at as u64, Some(TimestampStyle::RelativeTime)).mention();

            embed = embed.field(EmbedFieldBuilder::new(
                Lang::Fr.profile_joined_at(),
                format!("{timestamp_long} ({timestamp_relative})"),
            ));
        }

        let components = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: None,
                disabled: false,
                emoji: None,
                label: Some("Photo de profil".into()),
                style: ButtonStyle::Link,
                url: Some(avatar),
            })],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed.validate()?.build()])
            .components([components])
            .build();

        Ok(PostInChat::new(response, context.user.id, state).await?)
    }
}

/// Error when executing [`ProfileCommand`]
#[derive(Debug, Error)]
pub enum ProfileCommandError {
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("failed to build embed: {0}")]
    Embed(#[from] EmbedValidationError),
    #[error("failed to build image url: {0}")]
    ImageUrl(#[from] ImageSourceUrlError),
    #[error(transparent)]
    Redis(#[from] RedisClientError),
}

impl InteractionError for ProfileCommandError {
    const INTERACTION_NAME: &'static str = "profile";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            ProfileCommandError::Parse(error) => InteractionErrorKind::internal(error),
            ProfileCommandError::Embed(error) => InteractionErrorKind::internal(error),
            ProfileCommandError::ImageUrl(error) => InteractionErrorKind::internal(error),
            ProfileCommandError::Redis(error) => InteractionErrorKind::internal(error),
        }
    }
}
