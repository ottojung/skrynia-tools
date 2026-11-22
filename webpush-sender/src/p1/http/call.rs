//! Single bounded binary POST to the pinned trusted edge.
//! No redirects, no proxies, no cookies, hard deadlines, fail-closed statuses.
use super::{Edge, Error, IoFailure, Result};
use reqwest::blocking::{Client, Response};
use reqwest::{StatusCode, redirect};
use std::io::Read;
use std::time::Duration;

pub fn client(edge: &Edge) -> Result<Client> {
    Client::builder()
        .no_proxy()
        .redirect(redirect::Policy::none())
        // No `cookies` feature: reqwest cannot retain ambient cookie state.
        .http1_only()
        .connect_timeout(Duration::from_millis(edge.timeout_ms))
        .timeout(Duration::from_millis(edge.timeout_ms))
        .build()
        .map_err(|_| Error::Transport("edge-client"))
}

pub fn post(client: &Client, edge: &Edge, req: &[u8]) -> core::result::Result<Vec<u8>, IoFailure> {
    if req.is_empty() || req.len() > edge.max_request_bytes {
        return Err(IoFailure);
    }
    let response = client
        .post(edge.url())
        .header("Host", &edge.host)
        .header("Origin", &edge.origin)
        .header("X-Skrynia-Edge-Capability", &edge.capability)
        .header("Content-Type", "application/octet-stream")
        .body(req.to_vec())
        .send()
        .map_err(|_| IoFailure)?;
    read(response, edge.max_response_bytes)
}

fn read(response: Response, max: usize) -> core::result::Result<Vec<u8>, IoFailure> {
    let bound = u64::try_from(max).map_err(|_| IoFailure)?;
    if response.status() != StatusCode::OK {
        return Err(IoFailure);
    }
    if response.content_length().is_some_and(|n| n > bound) {
        return Err(IoFailure);
    }
    let mut body = Vec::new();
    response
        .take(bound.saturating_add(1))
        .read_to_end(&mut body)
        .map_err(|_| IoFailure)?;
    if body.is_empty() || body.len() > max {
        return Err(IoFailure);
    }
    Ok(body)
}
