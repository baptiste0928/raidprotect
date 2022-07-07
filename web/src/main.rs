use axum::{extract::Path, routing::get, Router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/:name", get(hello_name));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn hello_name(Path(name): Path<String>) -> String {
    format!("Hello, {}!", name)
}
