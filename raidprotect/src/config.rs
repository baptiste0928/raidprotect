//! Configuration model.
//!
//! Configuration is loaded at runtime from environment variables prefixed with
//! `RAIDPROTECT_`. If variables are defined in a `.env` file, they will take
//! precedence over other variables.

use raidprotect_util::logging::LogConfig;
use serde::Deserialize;

/// Parse configuration from environment variables.
pub fn parse_config() -> Result<Config, envy::Error> {
    dotenv::dotenv().ok();
    envy::prefixed("RAIDPROTECT_").from_env()
}

/// RaidProtect configuration model.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Discord bot token.
    pub token: String,
    /// ID of the guild in which commands should be created.
    ///
    /// This is useful when developing to reflect changes to commands instantly.
    /// If not set, commands will be created globally.
    ///
    /// **Warning:** if set, all previously created global commands will be
    /// removed to avoid duplicates. Do not enable this in production.
    pub command_guild: Option<u64>,
    /// Redis connection uri.
    ///
    /// The connection uri should use the `redis://` scheme. Defaults to
    /// `redis://localhost:6379` if missing.
    #[serde(default = "default_redis_uri")]
    pub redis_uri: String,
    /// MongoDB connection uri.
    ///
    /// The format of the connection string is described [here]. Defaults to
    /// `mongodb://localhost:27017` if missing.
    ///
    /// [here]: https://www.mongodb.com/docs/manual/reference/connection-string/#connection-string-formats
    #[serde(default = "default_mongodb_uri")]
    pub mongodb_uri: String,
    /// MongoDB database name.
    ///
    /// Defaults to `raidprotect` if missing.
    #[serde(default = "default_database")]
    pub mongodb_database: String,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: LogConfig,
}

/// Default Redis connection uri.
fn default_redis_uri() -> String {
    "redis://localhost:6379".to_string()
}

/// Default MongoDB connection uri.
fn default_mongodb_uri() -> String {
    "mongodb://localhost:27017".to_string()
}

/// Default database name.
fn default_database() -> String {
    "raidprotect".to_string()
}
