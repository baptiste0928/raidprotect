//! # RaidProtect Gateway.
//!
//! See `lib.rs` for more information.

use anyhow::{Context, Result};
use raidprotect_gateway::{cluster::ShardCluster, config::parse_config};
use raidprotect_transport::server::GatewayListener;
use raidprotect_util::shutdown::{wait_shutdown, Shutdown};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_config().context("Failed to load configuration")?;
    let _guard = config.log.init("gateway");
    let shutdown = Shutdown::new();

    // Initialize the cluster
    let cluster = ShardCluster::new(&config.token)
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
