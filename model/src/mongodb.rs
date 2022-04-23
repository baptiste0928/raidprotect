//! MongoDB client.
//!
//! This module provides a [`MongoDbClient`] type that wraps an underlying
//! MongoDB connection pool and exposes high-level methods to access data
//! stored in the database.

use std::time::Duration;

pub use mongodb::error::Error as MongoDbError;

use mongodb::{
    bson::{doc, to_bson, to_document},
    options, Client, Database,
};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use twilight_model::id::{marker::GuildMarker, Id};

use crate::{
    collection::{Guild, GUILDS_COLLECTION},
    serde::IdAsI64,
};

/// Wrapper around a MongoDB [`Client`].
///
/// The the [module documentation] to learn more. This type can be safely cloned
/// because the underlying [`Client`] uses `Arc`.
///
/// [module documentation]: super
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
        config.compressors = Some(vec![options::Compressor::Zstd { level: None }]);
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

        let guild = self
            .db()
            .collection::<Guild>(GUILDS_COLLECTION)
            .find_one(to_document(&query)?, None)
            .await?;

        Ok(guild)
    }

    /// Get the [`Guild`] for a given guild_id or create it with default configuration.
    pub async fn get_guild_or_create(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Guild, MongoDbError> {
        let query = GuildQuery { id: guild_id };
        let default_guild = Guild::new(guild_id);
        let options = options::FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(options::ReturnDocument::After)
            .build();

        let guild = self
            .db()
            .collection::<Guild>(GUILDS_COLLECTION)
            .find_one_and_update(
                to_document(&query)?,
                doc! { "$setOnInsert": to_bson(&default_guild)? },
                options,
            )
            .await?;

        Ok(guild.unwrap()) // SAFETY: upsert was set to true
    }

    /// Update or insert a [`Guild`] in the database.
    pub async fn update_guild(&self, guild: &Guild) -> Result<(), MongoDbError> {
        let query = GuildQuery { id: guild.id };
        let options = options::ReplaceOptions::builder().upsert(true).build();

        self.db()
            .collection::<Guild>(GUILDS_COLLECTION)
            .replace_one(doc! { "_id": to_document(&query)? }, guild, options)
            .await?;

        Ok(())
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
