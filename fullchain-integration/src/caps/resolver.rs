//! Host-to-global-IP mapping; the pinned IP is remapped to loopback only in
//! the injected push transport, never inside the SSRF guard itself.
use std::collections::HashMap;
use std::net::IpAddr;
use webpush_sender::error::IoFailure;
use webpush_sender::sub::Resolver;

pub struct FixedResolver(HashMap<String, Vec<IpAddr>>);

impl FixedResolver {
    pub fn new(host: &str, ip: IpAddr) -> Self {
        let mut m = HashMap::new();
        m.insert(host.to_owned(), vec![ip]);
        Self(m)
    }
}

impl Resolver for FixedResolver {
    fn resolve(&self, host: &str) -> core::result::Result<Vec<IpAddr>, IoFailure> {
        self.0.get(host).cloned().ok_or(IoFailure)
    }
}
