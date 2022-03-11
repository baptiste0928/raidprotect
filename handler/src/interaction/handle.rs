use std::sync::Arc;

use raidprotect_model::ClusterState;
use twilight_model::application::interaction::ApplicationCommand;

/// Handle incoming [`ApplicationCommand`]
///
/// This method will handle incoming commands depending on whereas they can run
/// on both dms and guilds, or only on guild.
pub async fn handle_command(_command: ApplicationCommand, _state: Arc<ClusterState>) {}
