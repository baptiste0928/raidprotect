use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::interaction::Interaction};
use twilight_util::builder::{embed::{EmbedBuilder}};

use crate::{impl_command_handle, cluster::ClusterState, interaction::{response::InteractionResponse, util::InteractionExt, embed::COLOR_TRANSPARENT}};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "help",
    desc = "Show information about RaidProtect",
    dm_permission = true
)]

pub struct HelpCommand;

impl_command_handle!(HelpCommand);

impl HelpCommand
{
    async fn exec(
        self,
        interaction: Interaction,
        _state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error>
    {
        let lang = interaction.locale()?;

        let embed = EmbedBuilder::new()
        .color(COLOR_TRANSPARENT)
        .title(lang.about_title())
        .description(lang.about_description());


        //PostInChat::create(response, author_id, state, lang).await
        Ok(InteractionResponse::EphemeralEmbed(embed.validate()?.build()))
    }
}
