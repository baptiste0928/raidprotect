use raidprotect_gateway::event::context::GuildContext;
use twilight_model::{
    application::{callback::CallbackData, interaction::Interaction},
    channel::embed::Embed, id::{marker::{ChannelMarker, InteractionMarker}, Id},
};
use twilight_util::builder::CallbackDataBuilder;

/// Handle incoming Discord [`Interaction`].
pub async fn handle(interaction: Interaction, ctx: GuildContext) {
    todo!()
}

/// Handle incoming [`ApplicationCommand`]
async fn handle_command() {}


/// Context of an [`ApplicationCommand`]
#[derive(Debug)]
pub struct CommandContext {
    /// ID of the interaction.
    id: Id<InteractionMarker>,
    /// Context of the guild the command was triggered from.
    guild: GuildContext,
    /// The channel the command was triggered from.
    channel_id: Id<ChannelMarker>,
}

/// Convert a type into [`CallbackData`].
///
/// This type is used for interaction responses.
pub trait IntoCallback {
    fn into_callback(self) -> CallbackData;
}

impl IntoCallback for CallbackData {
    fn into_callback(self) -> CallbackData {
        self
    }
}

impl IntoCallback for Embed {
    fn into_callback(self) -> CallbackData {
        CallbackDataBuilder::new().embeds([self]).build()
    }
}

impl<T: IntoCallback, E: IntoCallback> IntoCallback for Result<T, E> {
    fn into_callback(self) -> CallbackData {
        match self {
            Ok(value) => value.into_callback(),
            Err(error) => error.into_callback(),
        }
    }
}
