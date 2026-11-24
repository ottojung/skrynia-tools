//! Per-request client: no proxy, no redirects, no resolver other than the
//! single pinned override. Built fresh so no connection can be reused.
use super::trust::Trust;
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

pub const TOTAL_MS: u64 = 8_000;
const CONNECT_MS: u64 = 3_000;

pub fn client_for(
    trust: &Trust,
    host: &str,
    addr: SocketAddr,
    total_ms: u64,
) -> reqwest::Result<Client> {
    let builder = Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .dns_resolver(Arc::new(NoDns))
        .resolve(host, addr)
        .connect_timeout(Duration::from_millis(CONNECT_MS))
        .timeout(Duration::from_millis(total_ms));
    trust.apply(builder)?.build()
}

#[derive(Debug)]
struct NoDns;

impl reqwest::dns::Resolve for NoDns {
    fn resolve(&self, _name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        Box::pin(async { Err("dns disabled: pinned address required".into()) })
    }
}
