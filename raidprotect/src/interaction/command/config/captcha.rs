//! Captcha configuration commands.

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::InteractionChannel, guild::Role,
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "captcha", desc = "Configure the RaidProtect captcha")]
pub enum CaptchaConfigCommand {
    #[command(name = "enable")]
    Enable(CaptchaEnableCommand),
    #[command(name = "disable")]
    Disable(CaptchaDisableCommand),
    #[command(name = "logs")]
    Logs(CaptchaLogsCommand),
    #[command(name = "autorole-add")]
    AutoroleAdd(CaptchaAutoroleAddCommand),
    #[command(name = "autorole-remove")]
    AutoroleRemove(CaptchaAutoroleRemoveCommand),
    #[command(name = "autorole-list")]
    AutoroleList(CaptchaAutoroleListCommand),
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "enable", desc = "Enable the RaidProtect captcha")]
pub struct CaptchaEnableCommand;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "disable", desc = "Disable the RaidProtect captcha")]
pub struct CaptchaDisableCommand;

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "logs", desc = "Set the RaidProtect captcha logs channel")]
pub struct CaptchaLogsCommand {
    /// Channel to send the logs to.
    channel: InteractionChannel,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-add",
    desc = "Add a role to the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleAddCommand {
    /// Role to add to the autorole.
    role: Role,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-remove",
    desc = "Remove a role from the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleRemoveCommand {
    /// Role to remove from the autorole.
    role: Role,
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(
    name = "autorole-list",
    desc = "List the roles of the RaidProtect captcha autorole"
)]
pub struct CaptchaAutoroleListCommand;
