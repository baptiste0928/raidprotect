//! Profile command.

use std::time::Duration;

use thiserror::Error;
use tracing::instrument;
use twilight_embed_builder::{
    image_source::ImageSourceUrlError, EmbedBuilder, EmbedError, EmbedFieldBuilder, ImageSource,
};
use twilight_interactions::{
    command::{CommandModel, CreateCommand, ResolvedUser},
    error::ParseError,
};
use twilight_util::snowflake::Snowflake;

use crate::{
    embed::COLOR_TRANSPARENT,
    interaction::{
        context::CommandContext,
        response::{CommandResponse, InteractionError, InteractionErrorKind},
    },
    translations::Lang,
    util::{avatar_url, ImageFormat, ImageSize, TimestampStyle},
};

/// Profile command model.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "profile", desc = "Show information about a user profile")]
pub struct ProfileCommand {
    /// Mention or ID of the user.
    pub user: ResolvedUser,
}

impl ProfileCommand {
    /// Handle interaction for this command.
    #[instrument]
    pub async fn handle(context: CommandContext) -> Result<CommandResponse, ProfileCommandError> {
        let parsed = ProfileCommand::from_interaction(context.data.into())?;
        let user = parsed.user.resolved;

        let mut embed = EmbedBuilder::new()
            .color(COLOR_TRANSPARENT)
            .title(Lang::Fr.profile_title(user.discriminator(), &user.name))
            .thumbnail(ImageSource::url(avatar_url(
                &user,
                ImageFormat::Jpeg,
                ImageSize::Size1024,
            ))?);

        // User profile creation time.
        let created_at = Duration::from_millis(user.id.timestamp() as u64).as_secs();
        let timestamp_long = TimestampStyle::LongDate.format(created_at);
        let timestamp_relative = TimestampStyle::RelativeTime.format(created_at);

        embed = embed.field(EmbedFieldBuilder::new(
            Lang::Fr.profile_created_at(),
            format!("{timestamp_long} ({timestamp_relative})"),
        ));

        // Member join date.
        if let Some(member) = parsed.user.member {
            let joined_at = member.joined_at.as_secs();
            let timestamp_long = TimestampStyle::LongDate.format(joined_at as u64);
            let timestamp_relative = TimestampStyle::RelativeTime.format(joined_at as u64);

            embed = embed.field(EmbedFieldBuilder::new(
                Lang::Fr.profile_joined_at(),
                format!("{timestamp_long} ({timestamp_relative})"),
            ));
        }

        Ok(CommandResponse::Embed(embed.build()?))
    }
}

/// Error when executing [`ProfileCommand`]
#[derive(Debug, Error)]
pub enum ProfileCommandError {
    #[error("failed to parse command: {0}")]
    Parse(#[from] ParseError),
    #[error("failed to build embed: {0}")]
    Embed(#[from] EmbedError),
    #[error("failed to build image url: {0}")]
    ImageUrl(#[from] ImageSourceUrlError),
}

impl InteractionError for ProfileCommandError {
    const INTERACTION_NAME: &'static str = "profile";

    fn into_error(self) -> InteractionErrorKind {
        match self {
            ProfileCommandError::Parse(error) => InteractionErrorKind::internal(error),
            ProfileCommandError::Embed(error) => InteractionErrorKind::internal(error),
            ProfileCommandError::ImageUrl(error) => InteractionErrorKind::internal(error),
        }
    }
}
