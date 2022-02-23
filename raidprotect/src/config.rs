//! Configuration model.
//!
//! Configuration is loaded at runtime from a `Settings.toml` file
//! or environment prefixed with `RAIDPROTECT_`.

use raidprotect_util::logging::LogConfig;
use serde::Deserialize;

/// Parse configuration from `Settings.toml` or environment variables.
pub fn parse_config() -> Result<Config, envy::Error> {
    dotenv::dotenv().ok();
    envy::prefixed("RAIDPROTECT_").from_env()
}

/// RaidProtect configuration model.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Discord bot token.
    pub token: String,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: LogConfig,
}
