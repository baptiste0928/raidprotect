mod config;

use anyhow::{Result, Context};

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::parse_config().context("Failed to load configuration")?;
    let _guard = config.log.init("raidprotect");

    Ok(())
}
