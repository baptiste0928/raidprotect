use anyhow::Context;
use axum::{extract::Path, routing::get, Router};
use raidprotect_model::config::{parse_config, WebConfig};
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = parse_config::<WebConfig>().context("failed to load configuration")?;
    let _guard = config.log.init("raidprotect-web");

    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/:name", get(hello_name))
        // `TraceLayer` is provided by tower-http to trace http requests.
        .layer(TraceLayer::new_for_http());

    info!("listening on {}", &config.address);
    axum::Server::try_bind(&config.address)
        .context("failed to bind server address")?
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn hello_name(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}
