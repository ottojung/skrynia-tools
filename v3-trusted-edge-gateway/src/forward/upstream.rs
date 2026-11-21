use bytes::Bytes;
use futures_util::stream::{self, StreamExt, TryStreamExt};
use http_body_util::BodyExt;

pub async fn send(
    state: &crate::state::State,
    operator: [u8; 32],
    body: axum::body::Body,
    max: usize,
) -> Result<reqwest::Response, axum::http::StatusCode> {
    let limited = super::limit::LimitedBody::new(body, max);
    let data = limited
        .into_data_stream()
        .map_err(|_| std::io::Error::other("body read error"));
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
        .body(reqwest::Body::wrap_stream(stream))
        .send()
        .await
        .map_err(|_| axum::http::StatusCode::BAD_GATEWAY)
}
