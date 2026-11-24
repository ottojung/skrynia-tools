//! Test-only F2-F4 negatives for the loopback trusted-edge adapter.
//! Production controls in http/{endpoint,call}.rs are unchanged here.
use super::P1Transport;
use super::http::{CAP, HOST, HttpP1, MockEdge, ORIGIN, Reply, edge, res};

#[test]
fn malformed_edge_headers_and_cap_fail_at_construction() {
    for bad in ["a\r\nb", "a\nb", "a\x00b", "a\x7fb"] {
        let mut host = edge(9, HOST, ORIGIN, CAP);
        host.host = bad.to_owned();
        assert!(HttpP1::new(host).is_err());
        let mut cap = edge(9, HOST, ORIGIN, CAP);
        cap.capability = bad.to_owned();
        assert!(HttpP1::new(cap).is_err());
    }
    let mut short = edge(9, HOST, ORIGIN, CAP);
    short.capability = "short".to_owned();
    assert!(HttpP1::new(short).is_err());
    let mut long = edge(9, HOST, ORIGIN, CAP);
    long.capability = "x".repeat(257);
    assert!(HttpP1::new(long).is_err());
}

#[test]
fn ipv6_loopback_pinned_non_loopback_ipv6_rejected() {
    let mut local = edge(9, HOST, ORIGIN, CAP);
    local.ip = "::1".parse().unwrap();
    assert_eq!(local.url(), "http://[::1]:9/_skrynia/");
    assert!(HttpP1::new(local).is_ok());
    let mut foreign = edge(9, HOST, ORIGIN, CAP);
    foreign.ip = "2001:db8::1".parse().unwrap();
    assert!(HttpP1::new(foreign).is_err());
}

#[test]
fn redirect_with_location_is_not_followed() {
    let target = MockEdge::start(Reply::Ok(res(0, b"followed")), HOST, ORIGIN, CAP, 1);
    let first = MockEdge::start(Reply::Redirect(target.port), HOST, ORIGIN, CAP, 1);
    let io = HttpP1::new(edge(first.port, HOST, ORIGIN, CAP)).unwrap();
    assert!(io.call(&[20]).is_err());
    assert_eq!(first.seen().len(), 1);
    assert!(target.seen().is_empty(), "redirect must not be followed");
}
