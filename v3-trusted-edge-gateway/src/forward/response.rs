use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use futures_util::stream::{StreamExt, TryStreamExt};

pub fn from_upstream(upstream: reqwest::Response, max: usize) -> Response {
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
    let stream = upstream
        .bytes_stream()
        .scan(0usize, move |seen, item| {
            let max = max;
            let out = match item {
                Ok(bytes) => match seen.checked_add(bytes.len()) {
                    Some(v) if v <= max => {
                        *seen = v;
                        Some(Ok(bytes))
                    }
                    _ => Some(Err(())),
                },
                Err(_) => Some(Err(())),
            };
            futures_util::future::ready(out)
        })
        .map_err(|_| std::io::Error::other("upstream error"));
    let mut response = Response::new(Body::from_stream(stream));
    *response.status_mut() = status;
    *response.headers_mut() = headers;
    response
}
