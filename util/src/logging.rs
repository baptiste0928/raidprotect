//! Logging utility functions.
//!
//! This module contains logging functions used by binary crates of RaidProtect.
//! It allow to keep consistent logging strategy across all the crates.

use serde::{de, Deserialize};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

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
    /// This use [`tracing-subscriber::fmt`] for emitting
    /// logs in the terminal.
    Terminal,
    /// File output.
    ///
    /// This use [`tracing-appender`] and write log files
    /// in the folder configured in [`LogConfig`] (by
    /// default, a `log` folder in the current directory).
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
