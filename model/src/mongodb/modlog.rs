//! Models for the `modlogs` collection.

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DisplayFromStr};
use time::OffsetDateTime;
use twilight_model::{
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
    util::ImageHash,
};

use crate::serde::{DateTimeAsBson, IdAsI64};

/// Moderation log entry.
///
/// This struct represent a unique moderation log entry stored in the database.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Modlog {
    /// Unique ID of the moderation log.
    #[serde(rename = "_id")]
    pub id: Option<ObjectId>,
    /// Type of moderation log.
    pub kind: ModlogType,
    /// Guild where the moderation log was issued.
    #[serde_as(as = "IdAsI64")]
    pub guild_id: Id<GuildMarker>,
    /// User targeted by the moderation log.
    pub user: ModlogUser,
    /// Moderator that issued the moderation log.
    pub moderator: ModlogUser,
    /// Date of the moderation log.
    #[serde_as(as = "DateTimeAsBson")]
    pub date: OffsetDateTime,
    /// Optional reason provided by the moderator.
    pub reason: Option<String>,
    /// Optional notes attached to the moderation log.
    pub notes: Option<String>,
}

impl Modlog {
    /// Name of the MongoDB collection.
    pub const COLLECTION: &'static str = "modlogs";
}

/// Type of modlog entry.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ModlogType {
    Kick,
}

/// User model stored with modlog information.
///
/// This model is a simplified version of Discord user data that is stored with
/// moderation logs.
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ModlogUser {
    #[serde_as(as = "IdAsI64")]
    pub id: Id<UserMarker>,
    pub name: String,
    pub discriminator: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub avatar: Option<ImageHash>,
}
