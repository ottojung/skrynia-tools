//! Endpoint URLs are untrusted. Resolve once, check every answer, pin the IP.
use super::{Error, IoFailure, Result, host, ip};
use std::net::IpAddr;

pub trait Resolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub host: String,
    pub ip: IpAddr,
    pub port: u16,
    pub path: String,
}

pub fn validate_url(url: &str, resolver: &impl Resolver) -> Result<Target> {
    let rest = url.strip_prefix("https://").ok_or(Error::Ssrf("scheme"))?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, "/"),
    };
    if path.contains('#') || path.contains('\\') || path.bytes().any(|b| b < 0x20 || b == 0x7f) {
        return Err(Error::Ssrf("path"));
    }
    let auth = host::parse(authority)?;
    // An injected resolver I/O failure is a transient local failure, not an
    // unsafe answer. Keep Ssrf strictly for malformed URLs and bad addresses.
    let answers = resolver
        .resolve(&auth.host)
        .map_err(|_| Error::Transport("dns-io"))?;
    // Empty success means no usable answer; treat as transient, not unsafe.
    // Any non-global answer remains a hard SSRF denial.
    if answers.is_empty() {
        return Err(Error::Transport("dns-empty"));
    }
    if !answers.iter().copied().all(ip::is_global) {
        return Err(Error::Ssrf("address"));
    }
    Ok(Target {
        host: auth.host,
        ip: answers[0],
        port: auth.port,
        path: path.to_owned(),
    })
}
