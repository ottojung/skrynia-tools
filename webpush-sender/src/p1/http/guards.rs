//! Negative transport tests: bearer, host/origin, bounds, endpoint pinning.
use super::cases::{CAP, HOST, ORIGIN, edge, res};
use super::{HttpP1, MockEdge, P1Transport, Reply};

#[test]
fn invalid_edge_capability_is_rejected() {
    let mock = MockEdge::start(Reply::Ok(res(0, b"ok")), HOST, ORIGIN, CAP, 1);
    let io = HttpP1::new(edge(mock.port, HOST, ORIGIN, "wrong-edge-capability-0001")).unwrap();
    assert!(io.call(&[20]).is_err());
}

#[test]
fn wrong_host_and_origin_are_rejected() {
    let mock = MockEdge::start(Reply::Ok(res(0, b"ok")), HOST, ORIGIN, CAP, 2);
    let host = HttpP1::new(edge(mock.port, "evil.example", ORIGIN, CAP)).unwrap();
    assert!(host.call(&[20]).is_err());
    let origin = HttpP1::new(edge(mock.port, HOST, "https://evil.example", CAP)).unwrap();
    assert!(origin.call(&[20]).is_err());
}

#[test]
fn oversized_and_non_ok_statuses_fail_closed() {
    let mock = MockEdge::start(Reply::Oversized, HOST, ORIGIN, CAP, 1);
    let mut cfg = edge(mock.port, HOST, ORIGIN, CAP);
    cfg.max_response_bytes = 32;
    assert!(HttpP1::new(cfg).unwrap().call(&[20]).is_err());
    for status in [302, 401, 500] {
        let mock = MockEdge::start(Reply::Status(status), HOST, ORIGIN, CAP, 1);
        assert!(
            HttpP1::new(edge(mock.port, HOST, ORIGIN, CAP))
                .unwrap()
                .call(&[20])
                .is_err()
        );
    }
}

#[test]
fn only_pinned_loopback_endpoint_is_accepted() {
    let mut cfg = edge(1, HOST, ORIGIN, CAP);
    cfg.ip = "8.8.8.8".parse().unwrap();
    assert!(HttpP1::new(cfg).is_err());
    let cfg = edge(0, HOST, ORIGIN, CAP);
    assert!(HttpP1::new(cfg).is_err());
    let url = edge(9, HOST, ORIGIN, CAP).url();
    assert!(url.starts_with("http://127.0.0.1:9/"));
    assert!(url.ends_with("/_skrynia/"));
}
