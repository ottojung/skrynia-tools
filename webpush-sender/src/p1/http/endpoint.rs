//! Pinned loopback edge endpoint and its app-scoped bearer.
//! No DNS, no remote endpoint and no operator/P0 token are ever accepted here.
use super::{Error, Result};
use std::net::IpAddr;

pub const PATH: &str = "/_skrynia/";
const MAX_HEADER: usize = 256;
const MAX_BODY: usize = 16 * 1024 * 1024;

pub struct Edge {
    pub ip: IpAddr,
    pub port: u16,
    pub host: String,
    pub origin: String,
    pub capability: String,
    pub timeout_ms: u64,
    pub max_request_bytes: usize,
    pub max_response_bytes: usize,
}

impl Edge {
    pub fn validate(&self) -> Result<()> {
        if !self.ip.is_loopback() || self.port == 0 {
            return Err(Error::Ssrf("edge"));
        }
        for value in [&self.host, &self.origin, &self.capability] {
            if value.is_empty()
                || value.len() > MAX_HEADER
                || value.bytes().any(|b| b < 0x21 || b == 0x7f)
            {
                return Err(Error::Config("edge-header"));
            }
        }
        if self.capability.len() < 16 {
            return Err(Error::Config("edge-cap"));
        }
        let limits = [self.max_request_bytes, self.max_response_bytes];
        if self.timeout_ms == 0
            || self.timeout_ms > 60_000
            || limits.iter().any(|n| *n == 0 || *n > MAX_BODY)
        {
            return Err(Error::Config("edge-bound"));
        }
        Ok(())
    }

    pub fn url(&self) -> String {
        let host = match self.ip {
            IpAddr::V4(v) => v.to_string(),
            IpAddr::V6(v) => format!("[{v}]"),
        };
        format!("http://{host}:{}{PATH}", self.port)
    }
}
