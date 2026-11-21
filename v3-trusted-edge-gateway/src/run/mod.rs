mod server;
mod signal;

use std::sync::Arc;

pub async fn run() -> Result<(), String> {
    let path = std::env::var("SKRYNIA_EDGE_GATEWAY_CONFIG")
        .map_err(|_| "SKRYNIA_EDGE_GATEWAY_CONFIG is required")?;
    let config = crate::config::load(&path)?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "failed to build upstream client")?;
    let bind = config.bind.parse().map_err(|_| "invalid bind address")?;
    let state = crate::state::State {
        inner: Arc::new(crate::state::Inner { config, client }),
    };
    server::serve(bind, state, signal::shutdown()).await
}
