mod body;

use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use std::time::Duration;

pub fn from_upstream(upstream: reqwest::Response, max: usize, read_idle: Duration) -> Response {
    let status =
        StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut headers = HeaderMap::new();
    if let Some(ct) = upstream.headers().get(reqwest::header::CONTENT_TYPE) {
        headers.insert(axum::http::header::CONTENT_TYPE, ct.clone());
    }
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        axum::http::header::HeaderValue::from_static("no-store"),
    );
    let stream = body::limited_idle(upstream.bytes_stream(), max, read_idle);
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}
