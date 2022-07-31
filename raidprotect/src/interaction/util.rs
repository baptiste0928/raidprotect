//! Utility function to handle incoming interactions.
use std::{
    fmt::{self, Display},
    mem,
    str::FromStr,
};

use anyhow::{bail, Context};
use twilight_interactions::command::CommandModel;
use twilight_model::{
    application::interaction::{Interaction, InteractionData},
    guild::PartialMember,
    id::{marker::GuildMarker, Id},
};

use crate::translations::Lang;

/// Extension trait adding methods to [`Interaction`].
pub trait InteractionExt {
    /// Get the guild the interaction was triggered in.
    fn guild(&self) -> Result<GuildInteraction<'_>, anyhow::Error>;

    /// Get the user locale.
    fn locale(&self) -> Result<Lang, anyhow::Error>;
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

    fn locale(&self) -> Result<Lang, anyhow::Error> {
        let locale = self.locale.as_ref().context("missing locale")?;

        Ok(Lang::from(&**locale))
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

/// Component custom id.
///
/// This type is used to hold component identifiers, used in buttons or modals.
/// Each custom id must have a `name` which correspond to the component type,
/// and optionally an `id` used to store component state.
pub struct CustomId {
    /// Name of the component.
    pub name: String,
    /// ID of the component.
    pub id: Option<String>,
}

impl CustomId {
    /// Create a new custom id.
    pub fn new(name: impl Into<String>, id: String) -> Self {
        Self {
            name: name.into(),
            id: Some(id),
        }
    }
}

impl FromStr for CustomId {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            bail!("expected non-empty custom id");
        }

        match value.split_once(':') {
            Some((name, id)) => Ok(CustomId {
                name: name.to_string(),
                id: Some(id.to_string()),
            }),
            None => Ok(CustomId {
                name: value.to_string(),
                id: None,
            }),
        }
    }
}

impl Display for CustomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(id) = &self.id {
            f.write_str(&self.name)?;
            f.write_str(":")?;
            f.write_str(id)
        } else {
            f.write_str(&self.name)
        }
    }
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
        _ => bail!("unable to parse command data, received unknown data type"),
    };

    T::from_interaction(data.into()).context("failed to parse command data")
}

/// Implement `handle` method for a command type.
///
/// The generated method will parse the command from an interaction and execute
/// it. The command type must implement [`CommandModel`] and have an `exec`
/// method with the following signature:
///
/// `async fn exec(self, interaction: Interaction, state: &ClusterState) -> Result<InteractionResponse, anyhow::Error>`
#[macro_export]
macro_rules! impl_command_handle {
    ($name:path) => {
        impl $name {
            #[::tracing::instrument]
            pub async fn handle(
                mut interaction: ::twilight_model::application::interaction::Interaction,
                state: &$crate::cluster::ClusterState,
            ) -> Result<$crate::interaction::response::InteractionResponse, ::anyhow::Error> {
                let parsed =
                    $crate::interaction::util::parse_command_data::<Self>(&mut interaction)?;

                parsed.exec(interaction, state).await
            }
        }
    };
}

/// Generate a function that gives localized description or name text for a command.
///
/// The generated function respects `twilight_interactions::command` localization requirements.
/// It returns a `[(&'static str, &'static str); 3]` who implements `IntoIterator<Item = (ToString, ToString)>`
///
/// The name of the function needs to be a locale key (like `help_description`)
///
/// [Twilight-interactions Docs/ Command / Localization]
///
/// This macro needs to respect [Discord Docs/Locales] in locale names.
///
/// [Discord Docs/Locales]: https://discord.com/developers/docs/reference#locales
/// [Twilight-interactions Docs/ Command / Localization]: https://docs.rs/twilight-interactions/latest/twilight_interactions/command/index.html#localization
#[macro_export]
macro_rules! desc_localizations {
    ($name:ident) => {
        pub fn $name() -> [(&'static str, &'static str); 3] {
            [
                ("fr", $crate::translations::Lang::Fr.$name()),
                ("en-US", $crate::translations::Lang::En.$name()),
                ("en-GB", $crate::translations::Lang::En.$name()),
            ]
        }
    };
}
