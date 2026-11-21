use axum::{Router, routing::post};
use std::net::SocketAddr;

pub async fn serve(
    bind: SocketAddr,
    state: crate::state::State,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), String> {
    let app = Router::new()
        .route("/_skrynia/", post(crate::forward::handle))
        .route("/_skrynia", post(crate::forward::handle))
        .fallback(crate::forward::reject)
        .with_state(state);
    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .map_err(|e| format!("bind failed: {e}"))?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
        .map_err(|e| format!("server failed: {e}"))
}
