//! Configuration command.
//!
//! The configuration command allows the user to change the configuration of the
//! bot.

mod captcha;

pub use captcha::CaptchaConfigCommand;
use twilight_interactions::command::{CommandModel, CreateCommand};

/// Configuration command model.
///
/// This type is the main model that register all the configuration subcommands.
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "config", desc = "Configure RaidProtect on your server")]
pub enum ConfigCommand {
    #[command(name = "captcha")]
    Captcha(CaptchaConfigCommand),
}
