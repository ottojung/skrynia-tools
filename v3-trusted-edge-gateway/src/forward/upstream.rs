use bytes::Bytes;
use futures_util::stream::{self, StreamExt, TryStreamExt};
use http_body_util::BodyExt;
use std::time::Duration;

pub async fn send(
    state: &crate::state::State,
    operator: [u8; 32],
    body: axum::body::Body,
    max: usize,
) -> Result<reqwest::Response, axum::http::StatusCode> {
    let limited = super::limit::LimitedBody::new(body, max);
    let idle = super::idle::IdleBody::new(limited, state.upload_idle());
    let data = idle.into_data_stream().map_err(map_body_err);
    let prefix = Bytes::from([vec![0x01u8], operator.to_vec()].concat());
    let stream = stream::once(async move { Ok::<Bytes, std::io::Error>(prefix) }).chain(data);
    let url = state
        .inner
        .config
        .upstream
        .as_str()
        .parse::<reqwest::Url>()
        .map_err(|_| axum::http::StatusCode::BAD_GATEWAY)?;
    state
        .inner
        .client
        .post(url)
        .header("content-type", "application/octet-stream")
        .timeout(Duration::from_millis(
            state.inner.config.upstream_read_timeout_ms,
        ))
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await
        .map_err(map_send_err)
}

fn map_body_err(e: axum::Error) -> std::io::Error {
    let s = e.to_string();
    if s.contains("idle timeout") {
        std::io::Error::new(std::io::ErrorKind::TimedOut, s)
    } else {
        std::io::Error::other(s)
    }
}

fn map_send_err(e: reqwest::Error) -> axum::http::StatusCode {
    if e.is_timeout() {
        axum::http::StatusCode::GATEWAY_TIMEOUT
    } else {
        axum::http::StatusCode::BAD_GATEWAY
    }
}
