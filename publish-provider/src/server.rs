//! Loopback-only HTTP surface. Every syscall response is exactly four
//! little-endian bytes so the kernel can read a provider error code.

use super::abi;
use super::config::Config;
use super::fs;
use axum::body::Bytes;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::{Router, routing::get, routing::post};
use std::sync::{Arc, Mutex};

pub async fn run(config: Config) -> Result<(), String> {
    let state = Arc::new(fs::State {
        roots: config.roots,
        lock: Mutex::new(()),
    });
    let app = Router::new()
        .route("/syscalls_list", get(syscalls_list))
        .route("/publish", post(publish))
        .route("/unpublish", post(unpublish))
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(|e| format!("cannot bind {}: {e}", config.bind))?;
    axum::serve(listener, app)
        .await
        .map_err(|e| format!("provider serve failed: {e}"))
}

/// Kernel discovery: GET with no request body returns a plain JSON array.
async fn syscalls_list() -> Json<serde_json::Value> {
    Json(abi::syscalls_list())
}

async fn publish(AxumState(state): AxumState<Arc<fs::State>>, body: Bytes) -> impl IntoResponse {
    respond(&state, &body, true)
}

async fn unpublish(AxumState(state): AxumState<Arc<fs::State>>, body: Bytes) -> impl IntoResponse {
    respond(&state, &body, false)
}

fn respond(state: &Arc<fs::State>, body: &[u8], publish: bool) -> (StatusCode, Vec<u8>) {
    let code = match abi::decode(body) {
        Ok(ticket) => fs::call(state, ticket, publish),
        Err(code) => code,
    };
    (StatusCode::OK, abi::encode(code))
}
