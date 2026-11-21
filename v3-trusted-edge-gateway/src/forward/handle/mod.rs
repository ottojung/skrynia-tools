mod run;

use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::{Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::time::{Instant, timeout};

pub async fn handle(
    AxumState(state): AxumState<crate::state::State>,
    req: Request<Body>,
) -> Response {
    if req.method() != Method::POST {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let (parts, body) = req.into_parts();
    if let Err(status) = crate::auth::authenticate(&state, &parts.headers) {
        return status.into_response();
    }
    let acq = state.acquire_timeout();
    let req_to = state.request_timeout();
    let deadline = Instant::now() + req_to;
    let permit = match timeout(acq, state.inner.permits.clone().acquire_owned()).await {
        Ok(Ok(p)) => p,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let fwd = state.clone();
    match timeout(req_to, run::forward(fwd, parts, body, permit, deadline)).await {
        Ok(resp) => resp,
        Err(_) => StatusCode::GATEWAY_TIMEOUT.into_response(),
    }
}
