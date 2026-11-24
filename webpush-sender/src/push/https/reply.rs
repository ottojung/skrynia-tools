//! Bounded response handling. Header count, body bytes and Retry-After are
//! all capped; transport errors never carry remote text into the caller.
use super::{IoFailure, Reply};
use reqwest::blocking::Response;
use reqwest::header::RETRY_AFTER;
use std::io::Read;

const MAX_BODY: u64 = 64 * 1024;
const MAX_HEADERS: usize = 64;
const MAX_RETRY_MS: u64 = 60_000;

pub fn reply_for(response: Response) -> Result<Reply, IoFailure> {
    if response.headers().len() > MAX_HEADERS {
        return Err(IoFailure);
    }
    let status = response.status().as_u16();
    let retry_after_ms = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(retry_ms);
    let mut sink = Vec::new();
    response
        .take(MAX_BODY + 1)
        .read_to_end(&mut sink)
        .map_err(|_| IoFailure)?;
    if sink.len() as u64 > MAX_BODY {
        return Err(IoFailure);
    }
    Ok(Reply {
        status,
        retry_after_ms,
    })
}

fn retry_ms(value: &str) -> Option<u64> {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .map(|secs| secs.saturating_mul(1_000).min(MAX_RETRY_MS))
}
