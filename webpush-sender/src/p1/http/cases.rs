//! Shared fixtures: no real host, credential or subscriber data.
use super::{Config, Edge};

pub const HOST: &str = "long.a.vau.place";
pub const ORIGIN: &str = "https://long.a.vau.place";
pub const CAP: &str = "app-scoped-edge-capability-0001";
pub const SENTINEL: [u8; 32] = [0xA5; 32];

pub fn cfg() -> Config {
    Config {
        sender_cap: [7; 32],
        ns: [9; 32],
        key: b"notice/arrival".to_vec(),
        max_payload: 4096,
        max_attempts: 3,
        base_backoff_ms: 10,
        max_backoff_ms: 100,
        ttl_secs: 60,
        topic: [2; 16],
        vapid_sub: "mailto:ops@example.invalid".to_owned(),
        ledger_epoch: 0,
    }
}

pub fn edge(port: u16, host: &str, origin: &str, cap: &str) -> Edge {
    Edge {
        ip: "127.0.0.1".parse().unwrap(),
        port,
        host: host.to_owned(),
        origin: origin.to_owned(),
        capability: cap.to_owned(),
        timeout_ms: 2000,
        max_request_bytes: 8192,
        max_response_bytes: 4096,
    }
}

pub fn res(status: u8, body: &[u8]) -> Vec<u8> {
    let mut out = vec![1, status];
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(body);
    out
}
