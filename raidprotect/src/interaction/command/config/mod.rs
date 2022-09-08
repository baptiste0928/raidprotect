//! Configuration command.
//!
//! The configuration command allows the user to change the configuration of the
//! bot.

mod captcha;

pub use captcha::CaptchaConfigCommand;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::guild::Permissions;

use crate::{
    cluster::ClusterState,
    desc_localizations, impl_guild_command_handle,
    interaction::{response::InteractionResponse, util::GuildInteractionContext},
};

/// Configuration command model.
///
/// This type is the main model that register all the configuration subcommands.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "config",
    desc = "Configure RaidProtect on your server",
    desc_localizations = "config_description",
    default_permissions = "config_permissions"
)]
pub enum ConfigCommand {
    #[command(name = "captcha")]
    Captcha(CaptchaConfigCommand),
}

impl_guild_command_handle!(ConfigCommand);
desc_localizations!(config_description);

fn config_permissions() -> Permissions {
    Permissions::ADMINISTRATOR
}

impl ConfigCommand {
    async fn exec(
        self,
        ctx: GuildInteractionContext,
        state: &ClusterState,
    ) -> Result<InteractionResponse, anyhow::Error> {
        match self {
            Self::Captcha(command) => command.exec(ctx, state).await,
        }
    }
}
