//! Models for the `modlogs` collection.

use anyhow::anyhow;
use mongodb::{
    bson::{doc, oid::ObjectId, to_document, Bson},
    Cursor,
};
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

use super::DbClient;
use crate::serde::{DateTimeAsBson, IdAsI64};

/// Moderation log entry.
///
/// This type represent a moderation log entry stored in the `modlogs` collection
/// of the database.
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

// Implementation of methods to query the database.
impl DbClient {
    /// Insert a new [`Modlog`] in the database.
    pub async fn create_modlog(&self, modlog: &Modlog) -> Result<ObjectId, anyhow::Error> {
        let result = self
            .db()
            .collection::<Modlog>(Modlog::COLLECTION)
            .insert_one(modlog, None)
            .await?;

        match result.inserted_id {
            Bson::ObjectId(id) => Ok(id),
            other => Err(anyhow!("expected object id, got {:?}", other)),
        }
    }

    /// Get a [`Modlog`] from the database with its id.
    pub async fn get_modlog(&self, id: ObjectId) -> Result<Option<Modlog>, anyhow::Error> {
        let query = doc! { "_id": id };

        let modlog = self
            .db()
            .collection::<Modlog>(Modlog::COLLECTION)
            .find_one(query, None)
            .await?;

        Ok(modlog)
    }

    /// Find multiple [`Modlog`]s from the database that match a given guild id
    /// and optional user id.
    pub async fn find_modlogs(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Option<Id<UserMarker>>,
    ) -> Result<Cursor<Modlog>, anyhow::Error> {
        let query = ModlogQuery { guild_id, user_id };

        let cursor = self
            .db()
            .collection::<Modlog>(Modlog::COLLECTION)
            .find(to_document(&query)?, None)
            .await?;

        Ok(cursor)
    }
}

/// Query modlogs with guild_id and optional user_id
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct ModlogQuery {
    #[serde_as(as = "IdAsI64")]
    pub guild_id: Id<GuildMarker>,
    #[serde_as(as = "Option<IdAsI64>")]
    pub user_id: Option<Id<UserMarker>>,
}
