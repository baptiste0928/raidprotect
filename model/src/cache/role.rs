use serde::{Serialize, Deserialize};
use twilight_model::{id::{RoleId, GuildId}, guild::Permissions};

/// Cached model of a [`Role`].
///
/// This model is not cached within guilds to limit
/// data to send when requesting a [`CachedGuild`].
///
/// [`Role`]: twilight_model::guild::Role
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct CachedRole {
    /// Id of the role.
    pub id: RoleId,
    /// Id of the guild to which the role belongs.
    pub guild_id: GuildId,
    /// Name of the role.
    pub name: String,
    /// Color of the role.
    pub color: u32,
    /// Icon image hash.
    pub icon: Option<String>,
    /// Icon unicode emoji.
    ///
    /// This field is set if the role has an icon which is
    /// not a custom image but an existing unicode emoji.
    pub unicode_emoji: Option<String>,
    /// Position of the role.
    pub position: i64,
    /// Permissions of the role.
    pub permissions: Permissions,
    /// Whether the role is managed.
    ///
    /// Managed roles include bot, integration or boost roles.
    pub managed: bool,
}

/// Partial model of a [`Role`].
///
/// This type is used in [`CachedGuild`] and only contain fields
/// required for permissions calculation.
///
/// [`Role`]: twilight_model::guild::Role
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialRole {
    /// Position of the role.
    ///
    /// The position *should* be positive but can be negative
    /// in some cases. Only the ordering is important for
    /// permission calculations.
    pub position: i64,
    /// Permissions of the role.
    pub permissions: Permissions,
}
