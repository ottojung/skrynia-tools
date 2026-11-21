mod server;
mod signal;

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

pub async fn run() -> Result<(), String> {
    let path = std::env::var("SKRYNIA_EDGE_GATEWAY_CONFIG")
        .map_err(|_| "SKRYNIA_EDGE_GATEWAY_CONFIG is required")?;
    let config = crate::config::load(&path)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_millis(config.upstream_connect_timeout_ms))
        .build()
        .map_err(|_| "failed to build upstream client")?;
    let bind = config.bind.parse().map_err(|_| "invalid bind address")?;
    let permits = Semaphore::new(config.max_concurrent_requests);
    let state = crate::state::State {
        inner: Arc::new(crate::state::Inner {
            config,
            client,
            permits,
        }),
    };
    server::serve(bind, state, signal::shutdown()).await
}
