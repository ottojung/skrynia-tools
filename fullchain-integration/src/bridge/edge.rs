//! Loopback-only scoped edge endpoint; no operator token is accepted here.
use super::config::Config;
use std::net::{IpAddr, Ipv4Addr};
use webpush_sender::p1::Edge;

pub fn build(cfg: &Config) -> Edge {
    Edge {
        ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
        port: cfg.edge_port,
        host: cfg.edge_host.clone(),
        origin: cfg.edge_origin.clone(),
        capability: cfg.edge_cap.clone(),
        timeout_ms: 15_000,
        max_request_bytes: 2 * 1024 * 1024,
        max_response_bytes: 2 * 1024 * 1024,
    }
}
