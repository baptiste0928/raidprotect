//! RaidProtect Gateway.
//!
//! The gateway is the main component of the RaidProtect Discord bot.
//! It is responsible for the following :
//!
//! - Events from Discord are received and forwarded to appropriate services.
//! - An in-memory cache store information like guilds list and is exposed via
//!   RPC.
//! - The HTTP proxy provides global limiting for requests to the REST API.

mod cache;
mod cluster;
mod config;

use anyhow::{Context, Result};
use raidprotect_transport::server::GatewayListener;
use raidprotect_util::shutdown::{wait_shutdown, Shutdown};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::parse_config().context("Failed to load configuration")?;
    let _guard = config.log.init("gateway");
    let shutdown = Shutdown::new();

    // Initialize the cluster
    let cluster = cluster::ShardCluster::new(&config.token)
        .await
        .context("Failed to initialize shards cluster")?;

    // Start the gateway server and shards cluster
    let server = tokio::spawn(GatewayListener::start(config.port, shutdown.subscriber()));

    let cluster_run = tokio::spawn(cluster.start(shutdown.subscriber()));

    // Wait for shutdown
    tokio::select! {
        _ = server => (),
        _ = cluster_run => (),
        _ = wait_shutdown() => debug!("shutdown signal received")
    };

    info!("shutting down ...");
    shutdown.shutdown(5).await;

    Ok(())
}
