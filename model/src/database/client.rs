use std::time::Duration;

use mongodb::{bson::doc, options, Client, Database};

/// Wrapper around a MongoDB [`Client`].
///
/// This type wraps an underlying MongoDB connection pool and exposes high-level
/// methods to access data stored in the database.
///
/// It type can be cheaply cloned because the underlying [`Client`] uses [`Arc`].
///
/// [`Arc`]: std::sync::Arc
#[derive(Debug, Clone)]
pub struct DbClient {
    client: Client,
    database: String,
}

impl DbClient {
    /// Connects to a MongoDB database and returns the client.
    pub async fn connect(uri: &str, database: String) -> Result<Self, anyhow::Error> {
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
    pub async fn ping(&self) -> Result<(), anyhow::Error> {
        self.db().run_command(doc! { "ping": 1_i32 }, None).await?;

        Ok(())
    }
}
