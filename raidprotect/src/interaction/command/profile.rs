//! Profile command.
//!
//! This command shows basic information about a given user.

use std::time::Duration;

use anyhow::Context;
use twilight_interactions::command::{CommandModel, CreateCommand, ResolvedUser};
use twilight_mention::{
    timestamp::{Timestamp, TimestampStyle},
    Mention,
};
use twilight_model::application::{
    component::{button::ButtonStyle, ActionRow, Button, Component},
    interaction::Interaction,
};
use twilight_util::{
    builder::{
        embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder, ImageSource},
        InteractionResponseDataBuilder,
    },
    snowflake::Snowflake,
};

use crate::translations::Lang;
use crate::{
    cluster::ClusterState,
    desc_translation, impl_command_handle,
    interaction::{
        component::PostInChat, embed::COLOR_TRANSPARENT, response::InteractionResponse,
        util::InteractionExt,
    },
    util::resource::avatar_url,
};

/// Profile command model.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "profile",
    desc = "Show information about a user profile",
    desc_localizations = "profile_description",
    dm_permission = true
)]
pub struct ProfileCommand {
    /// Mention or ID of the user.
    pub user: ResolvedUser,
}

impl_command_handle!(ProfileCommand);
desc_translation!(profile_description);

impl ProfileCommand {
    async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        let user = self.user.resolved;
        let lang = interaction.locale()?;

        let avatar = avatar_url(&user, "jpg", 1024);
        let mut embed = EmbedBuilder::new()
            .color(COLOR_TRANSPARENT)
            .title(lang.profile_title(user.discriminator(), &user.name))
            .footer(EmbedFooterBuilder::new(format!("ID: {}", user.id)).build())
            .thumbnail(ImageSource::url(&avatar)?);

        // User profile creation time.
        let created_at = Duration::from_millis(user.id.timestamp() as u64).as_secs();
        let timestamp_long = Timestamp::new(created_at, Some(TimestampStyle::LongDate)).mention();
        let timestamp_relative =
            Timestamp::new(created_at, Some(TimestampStyle::RelativeTime)).mention();

        embed = embed.field(EmbedFieldBuilder::new(
            lang.profile_created_at(),
            format!("{timestamp_long} ({timestamp_relative})"),
        ));

        // Member join date.
        if let Some(member) = self.user.member {
            let joined_at = member.joined_at.as_secs();
            let timestamp_long =
                Timestamp::new(joined_at as u64, Some(TimestampStyle::LongDate)).mention();
            let timestamp_relative =
                Timestamp::new(joined_at as u64, Some(TimestampStyle::RelativeTime)).mention();

            embed = embed.field(EmbedFieldBuilder::new(
                lang.profile_joined_at(),
                format!("{timestamp_long} ({timestamp_relative})"),
            ));
        }

        let components = Component::ActionRow(ActionRow {
            components: vec![Component::Button(Button {
                custom_id: None,
                disabled: false,
                emoji: None,
                label: Some(lang.profile_avatar_button().into()),
                style: ButtonStyle::Link,
                url: Some(avatar),
            })],
        });

        let response = InteractionResponseDataBuilder::new()
            .embeds([embed.validate()?.build()])
            .components([components])
            .build();
        let author_id = interaction.author_id().context("missing author id")?;

        PostInChat::create(response, author_id, state, lang).await
    }
}
