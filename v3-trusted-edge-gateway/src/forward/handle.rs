use axum::body::Body;
use axum::extract::State as AxumState;
use axum::http::{Method, Request, StatusCode};
use axum::response::{IntoResponse, Response};

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
    match super::upstream::send(&state, operator, body, max).await {
        Ok(resp) => super::response::from_upstream(resp, max),
        Err(status) => status.into_response(),
    }
}
