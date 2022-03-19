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
    /// If not set, commands will be created globally.
    pub command_guild: Option<u64>,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: LogConfig,
}