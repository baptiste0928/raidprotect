use raidprotect_gateway::event::context::GuildContext;
use twilight_model::application::interaction::ApplicationCommand;

/// Handle incoming [`ApplicationCommand`]
///
/// This method will handle incoming commands depending on whereas they can run
/// on both dms and guilds, or only on guild.
pub async fn handle_command(interaction: ApplicationCommand, ctx: Option<GuildContext>) {}
