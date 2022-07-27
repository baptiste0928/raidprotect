use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{application::interaction::Interaction};
use twilight_util::builder::{embed::{EmbedBuilder, EmbedFooterBuilder, EmbedFieldBuilder}, InteractionResponseDataBuilder};

use crate::{impl_command_handle, cluster::ClusterState, interaction::{response::InteractionResponse, util::InteractionExt, embed::COLOR_TRANSPARENT, component::PostInChat}};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "about",
    desc = "Show information about RaidProtect",
    dm_permission = true
)]

pub struct AboutCommand;

impl_command_handle!(AboutCommand);

impl AboutCommand
{
    async fn exec(
        self,
        interaction: Interaction,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error>
    {
        let lang = interaction.locale()?;

        let embed = EmbedBuilder::new()
        .color(COLOR_TRANSPARENT)
        .title("📌 À propos de RaidProtect")
        //.description(lang.about_description())
        .field(EmbedFieldBuilder::new(
            "👋 Présentation",
            "RaidProtect est l'un des meilleurs bots anti-raid. Sa mission est d'empêcher les raids sur vos serveurs tout en étant accessible à tous.
            Pour cela, il dispose de plusieurs cordes à son arc. Il utilise notamment son anti-spam intelligent et son mode raid automatique.
            De plus, il dispose d'un captcha pour bloquer les selfbots avant même qu'ils aient accès à votre serveur.",
        ))
        .field(EmbedFieldBuilder::new(
            "🔎 Fonctionnalités",
            ":shield: Un anti-spam et anti-raid intelligent et réactif.
            :mag_right: Un captcha pour bloquer les selfbots.
            :construction: Un mode raid automatique et manuel, pour empêcher les utilisateurs de rejoindre votre serveur pendant un raid.
            :lock: Un système de verrouillage des salons.
            :tools: Une configuration et utilisation facile et intuitive.",
        ))
        .field(EmbedFieldBuilder::new(
            "📍 Liens",
            "
            [**Site Web**](https://echapp.com/)
            [**Documentation**](https://echapp.com/)
            [**Support**](https://echapp.com/)
            [**Github**](https://echapp.com/)",
        ))
        .footer(EmbedFooterBuilder::new(lang.about_footer()));


        //PostInChat::create(response, author_id, state, lang).await
        Ok(InteractionResponse::Embed(embed.validate()?.build()))
    }
}
