//! Configuration model.
//!
//! Configuration is loaded at runtime from a `Settings.toml` file
//! or environment prefixed with `GATEWAY_`.

use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use raidprotect_util::logging::LogConfig;
use serde::Deserialize;

/// Parse configuration from `Settings.toml` or environment variables.
pub fn parse_config() -> Result<Config, figment::Error> {
    Figment::new()
        .merge(Toml::file("Settings.toml"))
        .merge(Env::prefixed("GATEWAY_"))
        .extract()
}

/// Gateway configuration model.
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Discord bot token.
    pub token: String,
    /// Gateway server port.
    #[serde(default = "Config::default_port")]
    pub port: u16,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: LogConfig,
}

impl Config {
    fn default_port() -> u16 {
        4500
    }
}
