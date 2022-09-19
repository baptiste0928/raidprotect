//! # RaidProtect
//!
//! This crate is the binary of the RaidProtect Discord bot that link together
//! all other `raidprotect`-prefixed crates.
//!
//! ## Crates structure
//! - `cache`: custom cache that store Discord objects
//! - `event`: Discord event handlers
//! - `interaction`: interaction handlers
//! - `model`: models shared between crates
//! - `util`: contain utilities such as logging and shutdown

mod event;
mod feature;
mod interaction;
mod shard;
mod util;

use anyhow::{Context, Result};
use raidprotect_model::config::{parse_config, BotConfig};
use tokio::task;
use tracing::{debug, info};

use crate::util::shutdown::{wait_shutdown, Shutdown};

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_config::<BotConfig>().context("failed to load configuration")?;
    let log_config = config.log.clone();
    let _guard = log_config.init("raidprotect");

    // Initialize shard cluster
    let shutdown = Shutdown::new();
    let shards = shard::BotShards::new(config)
        .await
        .context("failed to start shard cluster")?;

    // Start the shards and wait for the shutdown signal
    //
    // Since the handler is `!Send` due to twilight's internal implementation,
    // we need to spawn the shards on the same thread as the main function.
    let local = task::LocalSet::new();

    local
        .run_until(async {
            let shards_handle = task::spawn_local(shards.handle(shutdown.subscriber()));

            tokio::select! {
                _ = shards_handle => (),
                _ = wait_shutdown() => debug!("shutdown signal received")
            };

            info!("shutting down ...");
            shutdown.shutdown(5).await;
        })
        .await;

    Ok(())
}

mod translations {
    //! Generated translations.
    //!
    //! This module contains translations generated with [`rosetta-i18n`].
    //!
    //! [`From<&str>`] is implemented to get the `Lang` corresponding to a given
    //! locale code. See [Discord Docs/Locales].
    //!
    //! [Discord Docs/Locales]: https://discord.com/developers/docs/reference#locales

    rosetta_i18n::include_translations!();

    impl Lang {
        /// Default language used when the user language is not supported.
        pub const DEFAULT: Self = Self::En;
    }

    impl From<&str> for Lang {
        fn from(value: &str) -> Self {
            let lang = match value.split_once('-') {
                Some((lang, _)) => lang,
                None => value,
            };

            match lang {
                "fr" => Lang::Fr,
                "en" => Lang::En,
                _ => Lang::DEFAULT,
            }
        }
    }
}
