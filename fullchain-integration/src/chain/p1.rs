//! HttpP1 transport bound to the real trusted edge.
use std::net::IpAddr;
use webpush_sender::p1::{Edge, HttpP1};

pub fn transport(edge: &super::EdgeProc) -> HttpP1 {
    let cfg = Edge {
        ip: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        port: edge.port,
        host: edge.host.clone(),
        origin: edge.origin.clone(),
        capability: edge.cap.clone(),
        timeout_ms: 10_000,
        max_request_bytes: 512 * 1024,
        max_response_bytes: 512 * 1024,
    };
    HttpP1::new(cfg).expect("edge cfg")
}
