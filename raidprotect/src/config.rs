//! Configuration model.
//!
//! Configuration is loaded at runtime from environment variables prefixed with
//! `RAIDPROTECT_`. Variables defined in a `.env` file are loaded before.

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
    /// MongoDB connection uri.
    ///
    /// The format of the connection string is described [here].
    ///
    /// [here]: https://www.mongodb.com/docs/manual/reference/connection-string/#connection-string-formats
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

/// Default database name.
fn default_database() -> String {
    "raidprotect".to_string()
}
