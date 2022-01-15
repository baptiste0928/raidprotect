//! # RaidProtect Gateway.
//!
//! See `lib.rs` for more information.

use std::sync::Arc;

use anyhow::{Context, Result};
use raidprotect_gateway::{cache::InMemoryCache, cluster::ShardCluster, config::parse_config};
use raidprotect_transport::server::GatewayListener;
use raidprotect_util::shutdown::{wait_shutdown, Shutdown};
use tracing::{debug, info};
use twilight_http::Client as HttpClient;

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_config().context("Failed to load configuration")?;
    let _guard = config.log.init("gateway");

    // Initialize HTTP client and get current user.
    let http = Arc::new(HttpClient::new(config.token.clone()));
    let current_user = http
        .current_user()
        .exec()
        .await
        .context("Failed to connect to the Discord API")?
        .model()
        .await?;

    info!(
        "Logged as {} with ID {}",
        current_user.name, current_user.id
    );

    let shutdown = Shutdown::new();
    let cache = Arc::new(InMemoryCache::new(current_user.id));

    // Initialize the cluster
    let cluster = ShardCluster::new(&config.token, http.clone(), cache.clone())
        .await
        .context("Failed to initialize shards cluster")?;

    // Start the gateway server and shards cluster
    let server = tokio::spawn(GatewayListener::start(
        config.port,
        cache.clone(),
        cluster.broadcast.clone(),
        shutdown.subscriber(),
    ));

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
