use axum::body::Body;
use axum::http::{StatusCode, request::Parts};
use axum::response::{IntoResponse, Response};
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::Instant;

pub async fn forward(
    state: crate::state::State,
    parts: Parts,
    body: Body,
    permit: OwnedSemaphorePermit,
    deadline: Instant,
) -> Response {
    let max = state.inner.config.max_body_bytes;
    if let Some(len) = parts
        .headers
        .get(axum::http::header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
    {
        if len > max {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }
    }
    let operator = match crate::token::load_operator_token(&state.inner.config.operator_token_file)
    {
        Ok(t) => t,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let idle = state.upstream_read_idle();
    match super::super::upstream::send(&state, operator, body, max).await {
        Ok(resp) => super::super::response::from_upstream(resp, max, idle, permit, deadline),
        Err(status) => status.into_response(),
    }
}
