//! Runtime configuration model.
//!
//! This module contains the models used to configure the binary crates. The
//! values are loaded at runtime from environment variables prefixed with
//! `RAIDPROTECT_` (see [`parse_config`]). These variables can be loaded from
//! a `.env` file.

use std::net::SocketAddr;

use serde::{de, Deserialize};

/// Parse configuration from environment variables.
///
/// Variables are loaded from `.env` file then parsed into the corresponding
/// type. See the [module documentation](self) for more information.
pub fn parse_config<T>() -> Result<T, envy::Error>
where
    T: de::DeserializeOwned,
{
    dotenv::dotenv().ok();
    envy::prefixed("RAIDPROTECT_").from_env()
}

/// Base bot configuration model.
#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    /// Discord bot token.
    pub token: String,
    /// Databases configuration.
    #[serde(flatten, default)]
    pub database: shared::DatabaseConfig,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: shared::LogConfig,
}

/// Base web api configuration model.
#[derive(Debug, Deserialize, Clone)]
pub struct WebConfig {
    /// Server listening address.
    #[serde(default = "default_address")]
    pub address: SocketAddr,
    /// Databases configuration.
    #[serde(flatten, default)]
    pub database: shared::DatabaseConfig,
    /// Logging configuration.
    #[serde(flatten, default)]
    pub log: shared::LogConfig,
}

/// Default server address.
fn default_address() -> SocketAddr {
    "127.0.0.1:3000".parse().unwrap()
}

/// Models shared by the base models.
pub mod shared {
    use serde::{de, Deserialize};
    use tracing::Level;
    use tracing_appender::non_blocking::WorkerGuard;

    /// Databases configuration model.
    ///
    /// This model holds configuration values for Redis and MongoDB database.
    #[derive(Debug, Deserialize, Clone)]
    #[serde(default)]
    pub struct DatabaseConfig {
        /// Redis connection uri.
        ///
        /// The connection uri should use the `redis://` scheme. Defaults to
        /// `redis://localhost:6379`.
        pub redis_uri: String,
        /// MongoDB connection uri.
        ///
        /// The format of the connection string is described [here]. Defaults to
        /// `mongodb://localhost:27017`.
        ///
        /// [here]: https://www.mongodb.com/docs/manual/reference/connection-string/#connection-string-formats
        pub mongodb_uri: String,
        /// MongoDB database name.
        ///
        /// Defaults to `raidprotect`.
        pub mongodb_database: String,
    }

    impl Default for DatabaseConfig {
        fn default() -> Self {
            Self {
                redis_uri: "redis://localhost:6379".to_string(),
                mongodb_uri: "mongodb://localhost:27017".to_string(),
                mongodb_database: "raidprotect".to_string(),
            }
        }
    }

    /// Logging configuration model.
    ///
    /// This model is used to parse logging configuration.
    #[derive(Debug, Deserialize, Clone)]
    #[serde(default)]
    pub struct LogConfig {
        /// Logger used to emit logs.
        pub log_type: LogKind,
        /// Max level of emitted logs.
        #[serde(deserialize_with = "deserialize_level")]
        pub log_level: Level,
        /// Folder used to store logs with [`LogKind::File`].
        pub log_folder: String,
    }

    impl LogConfig {
        /// Init logger depending on the configured [`LogKind`].
        ///
        /// The returned [`WorkerGuard`] must be stored in a variable
        /// and dropped when the main process exists. This ensure that
        /// all remaining logs are written when using [`LogKind::File`].
        pub fn init(&self, name: impl AsRef<str>) -> Option<WorkerGuard> {
            match self.log_type {
                LogKind::Terminal => self.init_terminal(),
                LogKind::File => self.init_file(name.as_ref()),
                LogKind::None => None,
            }
        }

        /// Init logger with [`LogKind::Terminal`].
        fn init_terminal(&self) -> Option<WorkerGuard> {
            tracing_subscriber::fmt()
                .compact()
                .with_max_level(self.log_level)
                .init();

            None
        }

        /// Init logger with [`LogKind::File`].
        ///
        /// The returned [`WorkerGuard`] must be dropped when the main process
        /// exits to ensure all logs are written in the file.
        fn init_file(&self, name: &str) -> Option<WorkerGuard> {
            let appender = tracing_appender::rolling::hourly(&self.log_folder, name);
            let (writer, guard) = tracing_appender::non_blocking(appender);

            tracing_subscriber::fmt()
                .compact()
                .with_max_level(self.log_level)
                .with_writer(writer)
                .with_ansi(false)
                .init();

            Some(guard)
        }
    }

    impl Default for LogConfig {
        fn default() -> Self {
            Self {
                log_type: LogKind::Terminal,
                log_level: Level::INFO,
                log_folder: "log".into(),
            }
        }
    }

    /// Type of logger used to emit logs.
    #[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    pub enum LogKind {
        /// Terminal output.
        ///
        /// This use [`mod@tracing_subscriber::fmt`] for emitting
        /// logs in the terminal.
        Terminal,
        /// File output.
        ///
        /// This use [`tracing_appender`] and write log files
        /// in the folder configured in [`LogConfig`] (by
        /// default, a `log` folder in the current directory).
        ///
        ///
        File,
        /// Disable logging.
        ///
        /// This is the default behavior if no logger is
        /// enabled.
        None,
    }

    fn deserialize_level<'de, D>(deserializer: D) -> Result<Level, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}
