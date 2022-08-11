use std::time::Duration;

use anyhow::{anyhow, Context};
use mongodb::{
    bson::{doc, oid::ObjectId, to_bson, to_document, Bson},
    options, Client, Cursor, Database,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::id::{
    marker::{GuildMarker, UserMarker},
    Id,
};

use super::{guild::Guild, modlog::Modlog};
use crate::serde::IdAsI64;

/// Error type returned by [`MongoDbClient`].
///
/// This type is a re-export of the error of `mongodb` crate.
pub type MongoDbError = mongodb::error::Error;

/// Wrapper around a MongoDB [`Client`].
///
/// This type wraps an underlying MongoDB connection pool and exposes high-level
/// methods to access data stored in the database.This type can be safely cloned
/// because the underlying [`Client`] uses `Arc`.
#[derive(Debug, Clone)]
pub struct MongoDbClient {
    client: Client,
    database: String,
}

impl MongoDbClient {
    /// Connects to a MongoDB database and returns the client.
    pub async fn connect(uri: &str, database: String) -> Result<Self, MongoDbError> {
        let mut config = options::ClientOptions::parse(uri).await?;

        // Set default configuration options
        config.app_name = Some(config.app_name.unwrap_or_else(|| "raidprotect".to_string()));
        config.connect_timeout = Some(Duration::from_secs(2));
        config.server_selection_timeout = Some(Duration::from_secs(2));
        config.compressors = Some(vec![options::Compressor::Zlib { level: None }]);
        config.default_database = Some(database.clone());

        let client = Client::with_options(config)?;
        Ok(Self { client, database })
    }

    /// Return a clone of the underlying client.
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    /// Returns a new [`Database`] for the connected database.
    pub fn db(&self) -> Database {
        self.client.database(&self.database)
    }

    /// Run a `ping` command to check if the database is connected.
    pub async fn ping(&self) -> Result<(), MongoDbError> {
        self.db().run_command(doc! { "ping": 1_i32 }, None).await?;

        Ok(())
    }

    /// Get the [`Guild`] for a given guild_id, if it exists.
    pub async fn get_guild(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Option<Guild>, MongoDbError> {
        let query = GuildQuery { id: guild_id };

        self.db()
            .collection::<Guild>(Guild::COLLECTION)
            .find_one(to_document(&query)?, None)
            .await
    }

    /// Get the [`Guild`] for a given guild_id or create it with default configuration.
    pub async fn get_guild_or_create(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Guild, anyhow::Error> {
        let query = GuildQuery { id: guild_id };
        let default_guild = Guild::new(guild_id);
        let options = options::FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(options::ReturnDocument::After)
            .build();

        let guild = self
            .db()
            .collection::<Guild>(Guild::COLLECTION)
            .find_one_and_update(
                to_document(&query)?,
                doc! { "$setOnInsert": to_bson(&default_guild)? },
                options,
            )
            .await?;

        guild.context("no guild sent by the database")
    }

    /// Update or insert a [`Guild`] in the database.
    pub async fn update_guild(&self, guild: &Guild) -> Result<(), MongoDbError> {
        let query = GuildQuery { id: guild.id };
        let options = options::ReplaceOptions::builder().upsert(true).build();

        self.db()
            .collection::<Guild>(Guild::COLLECTION)
            .replace_one(to_document(&query)?, guild, options)
            .await?;

        Ok(())
    }

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
    pub async fn get_modlog(&self, id: ObjectId) -> Result<Option<Modlog>, MongoDbError> {
        let query = doc! { "_id": id };

        self.db()
            .collection::<Modlog>(Modlog::COLLECTION)
            .find_one(query, None)
            .await
    }

    /// Find multiple [`Modlog`]s from the database that match a given guild id
    /// and optional user id.
    pub async fn find_modlogs(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Option<Id<UserMarker>>,
    ) -> Result<Cursor<Modlog>, MongoDbError> {
        let query = ModlogQuery { guild_id, user_id };

        self.db()
            .collection::<Modlog>(Modlog::COLLECTION)
            .find(to_document(&query)?, None)
            .await
    }
}

/// Query a guild with its guild_id
#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct GuildQuery {
    #[serde_as(as = "IdAsI64")]
    #[serde(rename = "_id")]
    pub id: Id<GuildMarker>,
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
