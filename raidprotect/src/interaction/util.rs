//! Utility function to handle incoming interactions.
use std::mem;

use anyhow::{bail, Context};
use twilight_interactions::command::CommandModel;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    guild::PartialMember,
    id::{marker::GuildMarker, Id},
    user::User,
};

/// Extension trait adding methods to [`Interaction`].
pub trait InteractionExt {
    /// Get the guild the interaction was triggered in.
    fn guild(&self) -> Result<GuildInteraction<'_>, anyhow::Error>;
}

impl InteractionExt for Interaction {
    fn guild(&self) -> Result<GuildInteraction<'_>, anyhow::Error> {
        let id = self
            .guild_id
            .context("interaction not executed in a guild")?;
        let member = self
            .member
            .as_ref()
            .context("missing interaction member data")?;

        Ok(GuildInteraction { id, member })
    }
}

/// Data for interactions triggered in a guild.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuildInteraction<'a> {
    /// ID of the guild.
    pub id: Id<GuildMarker>,
    /// The member that triggered the command.
    pub member: &'a PartialMember,
}

/// Parse incoming [`ApplicationCommand`] or [`ApplicationCommandAutocomplete`]
/// interactions into typed struct.
///
/// This takes a mutable [`Interaction`] since the inner [`CommandData`] is
/// replaced with [`None`] to avoid useless clones.
///
/// [`ApplicationCommand`]: twilight_model::application::interaction::InteractionType::ApplicationCommand
/// [`ApplicationCommandAutocomplete`]: twilight_model::application::interaction::InteractionType::ApplicationCommandAutocomplete
/// [`CommandData`]: twilight_model::application::interaction::application_command::CommandData
pub fn parse_command_data<T>(interaction: &mut Interaction) -> Result<T, anyhow::Error>
where
    T: CommandModel,
{
    let data = match mem::take(&mut interaction.data) {
        Some(InteractionData::ApplicationCommand(data)) => *data,
        other => bail!("unable to parse command data, received unknown data type"),
    };

    Ok(T::from_interaction(data.into()).context("failed to parse command data")?)
}
