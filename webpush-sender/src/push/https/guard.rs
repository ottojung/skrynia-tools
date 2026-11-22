//! Fail-closed precondition checks before any socket is opened.
use super::{PushRequest, is_global};

/// Address policy for the pinned IP. Production always requires a global
/// unicast pin; the feature-gated loopback fixture may pin a local address.
#[derive(Clone, Copy)]
pub enum IpPolicy {
    GlobalOnly,
    #[cfg(feature = "test-transport")]
    Fixture,
}

pub fn valid(req: &PushRequest, policy: IpPolicy) -> bool {
    let ok = req.port == 443
        && !req.host.is_empty()
        && req.host == req.host.to_ascii_lowercase()
        && req.path.starts_with('/')
        && !req
            .path
            .bytes()
            .any(|b| b < 0x20 || b == 0x7f || b == b'#' || b == b'\\');
    ok && match policy {
        IpPolicy::GlobalOnly => is_global(req.ip),
        #[cfg(feature = "test-transport")]
        IpPolicy::Fixture => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn req(ip: IpAddr) -> PushRequest {
        PushRequest {
            host: "push.example.test".into(),
            ip,
            port: 443,
            path: "/x".into(),
            authorization: "vapid t=t, k=k".into(),
            ttl: 60,
            topic: [0; 16],
            body: vec![1],
        }
    }

    #[test]
    fn non_global_pinned_ip_rejected_at_guard() {
        let local = req(IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert!(!valid(&local, IpPolicy::GlobalOnly));
        #[cfg(feature = "test-transport")]
        assert!(valid(&local, IpPolicy::Fixture));
        let global = req(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)));
        assert!(valid(&global, IpPolicy::GlobalOnly));
    }
}
