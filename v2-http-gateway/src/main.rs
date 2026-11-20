mod client;
mod config;
mod handler;
mod headers;
#[cfg(test)]
mod tests;
mod token;
mod upstream;
use std::{path::PathBuf, sync::Arc};
struct State {
    endpoint: reqwest::Url,
    token_path: PathBuf,
    client: reqwest::Client,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::Config::load()?;
    let state = Arc::new(State {
        endpoint: cfg.endpoint,
        token_path: cfg.token_path,
        client: client::build()?,
    });
    let app = axum::Router::new()
        .fallback(handler::handle)
        .layer(axum::extract::DefaultBodyLimit::max(8 * 1024 * 1024))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
