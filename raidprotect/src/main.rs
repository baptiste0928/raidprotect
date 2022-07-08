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

mod cluster;
mod event;
mod interaction;
mod util;

use anyhow::{Context, Result};
use raidprotect_model::config::{parse_config, BotConfig};
use tracing::{debug, info};

use crate::util::shutdown::{wait_shutdown, Shutdown};

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_config::<BotConfig>().context("failed to load configuration")?;
    let log_config = config.log.clone();
    let _guard = log_config.init("raidprotect");

    // Initialize shard cluster
    let shutdown = Shutdown::new();
    let cluster = cluster::ShardCluster::new(config)
        .await
        .context("failed to start shard cluster")?;

    // Start the shard cluster
    let cluster_run = tokio::spawn(cluster.start(shutdown.subscriber()));
    info!("started shard cluster");

    // Wait for shutdown
    tokio::select! {
        _ = cluster_run => (),
        _ = wait_shutdown() => debug!("shutdown signal received")
    };

    info!("shutting down ...");
    shutdown.shutdown(5).await;

    Ok(())
}

mod translations {
    //! Generated translations.
    //!
    //! This module contains translations generated with [`rosetta-i18n`].

    rosetta_i18n::include_translations!();
}
