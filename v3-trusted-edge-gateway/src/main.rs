mod app;
mod auth;
mod config;
mod forward;
mod token;

use axum::{Router, routing::post};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Clone)]
struct State {
    inner: Arc<Inner>,
}

struct Inner {
    config: config::Config,
    client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let path = std::env::var("SKRYNIA_EDGE_GATEWAY_CONFIG")
        .map_err(|_| "SKRYNIA_EDGE_GATEWAY_CONFIG is required")?;
    let config = config::Config::load(&path)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "failed to build upstream client")?;
    let bind: SocketAddr = config.bind.parse().map_err(|_| "invalid bind address")?;
    let state = State {
        inner: Arc::new(Inner { config, client }),
    };
    let app = Router::new()
        .route("/_skrynia/", post(forward::handle))
        .route("/_skrynia", post(forward::handle))
        .fallback(forward::reject)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|e| format!("bind failed: {e}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .map_err(|e| format!("server failed: {e}"))
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
