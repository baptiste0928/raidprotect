//! Event models.
//!
//! This module contains models representing Discord events received by theg
//! gateway. They match [`twilight_model::gateway::event`] models with additional
//! fields for cached resources.
//!
//! The gateway only send events required by consumers to avoid using unneeded
//! resources.

use serde::{Deserialize, Serialize};
use twilight_model::application::interaction::Interaction;

use crate::cache::CachedGuild;

/// Event received from Discord.
///
/// This type contain all events that can be received from Discord and handled
/// by the bot. Only event types that are processed are included.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Event {
    InteractionCreate(InteractionCreate),
}

/// Interaction create event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InteractionCreate {
    /// The cached guild if interaction was run in a guild.
    pub guild: Option<CachedGuild>,
    /// The created interaction.
    pub interaction: Interaction,
}
