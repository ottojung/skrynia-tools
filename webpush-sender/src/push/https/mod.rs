//! Pinned-IP HTTPS transport. TCP connects only to the Target-validated
//! address while SNI, Host and certificate identity remain the target host.
mod client;
mod guard;
mod reply;
mod request;
mod trust;

use super::transport::{PushTransport, Reply};
use super::{IoFailure, PushRequest, is_global};
use client::client_for;
use reply::reply_for;
use request::build;
use std::net::SocketAddr;
use trust::Trust;

/// Production-bound adapter: system roots, port 443, short fixed timeouts.
pub struct HttpsTransport {
    trust: Trust,
    connect_port: u16,
    timeout_ms: u64,
    ip_policy: guard::IpPolicy,
}

impl HttpsTransport {
    pub fn production() -> Self {
        Self {
            trust: Trust::production(),
            connect_port: 443,
            timeout_ms: client::TOTAL_MS,
            ip_policy: guard::IpPolicy::GlobalOnly,
        }
    }

    /// Loopback fixture only, behind the non-default `test-transport` feature.
    /// Operator and production configuration cannot reach this trust-widening
    /// constructor; they must use `production()` with built-in webpki roots.
    #[cfg(feature = "test-transport")]
    pub fn for_test(ca_der: &[u8], connect_port: u16, timeout_ms: u64) -> Self {
        Self {
            trust: Trust::test(ca_der),
            connect_port,
            timeout_ms,
            ip_policy: guard::IpPolicy::Fixture,
        }
    }
}

impl PushTransport for HttpsTransport {
    fn send(&self, req: &PushRequest) -> Result<Reply, IoFailure> {
        if !guard::valid(req, self.ip_policy) {
            return Err(IoFailure);
        }
        let addr = SocketAddr::new(req.ip, self.connect_port);
        let client =
            client_for(&self.trust, &req.host, addr, self.timeout_ms).map_err(|_| IoFailure)?;
        let response = client.execute(build(req)?).map_err(|_| IoFailure)?;
        reply_for(response)
    }
}
