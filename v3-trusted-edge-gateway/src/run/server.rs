use axum::{Router, routing::post};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use hyper_util::service::TowerToHyperService;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, time::timeout};

pub async fn serve(
    bind: SocketAddr,
    state: crate::state::State,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), String> {
    let max = Duration::from_millis(
        state.inner.config.request_timeout_ms + state.inner.config.acquire_timeout_ms + 500,
    );
    let app = Router::new()
        .route("/_skrynia/", post(crate::forward::handle))
        .route("/_skrynia", post(crate::forward::handle))
        .fallback(crate::forward::reject)
        .with_state(state);
    let l = TcpListener::bind(bind)
        .await
        .map_err(|e| format!("bind failed: {e}"))?;
    accept(l, app, max, shutdown).await
}
async fn accept(
    l: TcpListener,
    app: Router,
    max: Duration,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
) -> Result<(), String> {
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            r = l.accept() => {
                let (s, _) = r.map_err(|e| format!("accept failed: {e}"))?;
                let svc = TowerToHyperService::new(app.clone());
                tokio::spawn(async move {
                    let mut b = Builder::new(TokioExecutor::new());
                    b.http1().keep_alive(false);
                    let _ = timeout(max, b.serve_connection(TokioIo::new(s), svc)).await;
                });
            }
        }
    }
}
