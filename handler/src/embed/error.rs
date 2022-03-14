//! Error embeds.

use twilight_embed_builder::{EmbedBuilder, EmbedFooterBuilder};
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

/// Unknown command received
pub fn unknown_command() -> Embed {
    EmbedBuilder::new()
        .title("Cette commande n'est pas encore disponible")
        .color(COLOR_RED)
        .description(
            "La commande que vous essayez d'effectuer n'est pas encore \
            disponible. Patientez quelques minutes et réessayez.",
        )
        .build()
        .unwrap()
}

/// Command not available in direct messages
pub fn guild_only() -> Embed {
    EmbedBuilder::new()
        .title("Cette commande ne fonctionne pas en messages privés")
        .color(COLOR_RED)
        .description(
            "La commande que vous essayez d'utiliser doit obligatoirement être \
            appelée depuis un serveur Discord. Invitez RaidProtect sur votre \
            serveur pour pouvoir l'utiliser.",
        )
        .build()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error() {
        internal_error();
    }

    #[test]
    fn test_unknown_command() {
        unknown_command();
    }

    #[test]
    fn test_guild_only() {
        guild_only();
    }
}
