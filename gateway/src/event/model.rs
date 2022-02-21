//! Event handling models.
//!
//! This module expose models used to process incoming events.

use raidprotect_cache::model::CachedGuild;
use twilight_model::application::interaction::Interaction;

/// Event received from Discord.
///
/// This type contain all events that can be received from Discord and handled
/// by the bot. Only event types that are processed are included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    InteractionCreate(InteractionCreate),
}

/// Interaction create event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractionCreate {
    /// The cached guild if interaction was run in a guild.
    pub guild: Option<CachedGuild>,
    /// The created interaction.
    pub interaction: Interaction,
}
