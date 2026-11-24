//! Production constructor refuses non-443 ports and needs no injected trust.
use super::probe::request;
use std::net::{IpAddr, Ipv4Addr};
use webpush_sender::push::{HttpsTransport, PushTransport};

pub fn production() {
    let req = request(IpAddr::V4(Ipv4Addr::LOCALHOST), 8443);
    assert!(HttpsTransport::production().send(&req).is_err());
    let mut req = request(IpAddr::V4(Ipv4Addr::LOCALHOST), 443);
    req.host = "Push.Example.Test".into();
    assert!(HttpsTransport::production().send(&req).is_err());
    req.host = "no-such-host.invalid".into();
    assert!(HttpsTransport::production().send(&req).is_err());
}
