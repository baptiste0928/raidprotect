//! Partial models of cached data.
//!
//! This module expose partial models that only contain required fields
//! for permissions calculation. These models are used to improve performance
//! by avoiding sending unecessary data to clients.
//!
//! The [`IntoPartial`] can be used to create partial models from regular ones.

use serde::{Deserialize, Serialize};
use twilight_model::{
    channel::permission_overwrite::PermissionOverwrite, guild::Permissions, id::ChannelId,
};

/// Convert into a partial model.
pub trait IntoPartial: Sized {
    type Partial: Sized;

    /// Convert this type into a partial model.
    ///
    /// This method take a `&self` to only allocate necessary
    /// variables. The method should take care to clone values
    /// when required.
    fn into_partial(&self) -> Self::Partial;
}

/// Partial model of a [`Role`].
///
/// This type only contain fields required for permissions calculation.
///
/// [`Role`]: twilight_model::guild::Role
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
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

/// Partial model of a [`GuildChannel`].
///
/// This type only contain fields required for permissions calculation.
/// Only text channels and threads are cached as the bot does not interact
/// with voice channels.
///
/// [`GuildChannel`]: twilight_model::channel::GuildChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum PartialChannel {
    /// Partial text channel.
    Text(PartialTextChannel),
    /// Partial category channel.
    Category(PartialCategoryChannel),
    /// Partial thread.
    Thread(PartialThread),
}

/// Partial model of a [`TextChannel`].
///
/// [`TextChannel`]: twilight_model::channel::TextChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialTextChannel {
    /// If the channel is in a category, the category id.
    pub parent_id: Option<ChannelId>,
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

/// Partial model of a [`CategoryChannel`].
///
/// [`CategoryChannel`]: twilight_model::channel::CategoryChannel
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialCategoryChannel {
    /// Permission overwrites of the channel.
    pub permission_overwrites: Vec<PermissionOverwrite>,
}

/// Cached model of a [`PublicThread`] or [`PrivateThread`].
///
/// [`PublicThread`]: twilight_model::channel::thread::PublicThread
/// [`PrivateThread`]: twilight_model::channel::thread::PrivateThread
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PartialThread {
    /// Parent channel of the thread.
    pub parent_id: Option<ChannelId>,
}
