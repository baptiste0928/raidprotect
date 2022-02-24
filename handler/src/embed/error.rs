//! Error embeds.

use twilight_embed_builder::{EmbedBuilder, EmbedError, EmbedFooterBuilder};
use twilight_model::channel::embed::Embed;

use super::COLOR_RED;

/// Internal error embed
pub fn internal_error() -> Embed {
    EmbedBuilder::new()
        .title("Oups, une erreur inconnue s'est produite ...")
        .color(COLOR_RED)
        .description(
            "La commande que vous avez effectuée a renvoyé un \
                résultat imprévu. Pas de panique, nous avons été informés du \
                problème ! En attendant, veuillez réessayer la commande de \
                nouveau.\n\n\
                **Si le problème persiste, merci de nous en informer.** Vous \
                pouvez nous contacter en [rejoignant notre serveur Discord]\
                (https://discord.gg/raidprotect).",
        )
        .footer(EmbedFooterBuilder::new(
            "Okay, Houston, I believe we've had a problem here ...",
        ))
        .build()
        .unwrap()
}
