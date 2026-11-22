//! The pinned address, not any ambient resolver, decides the TCP destination.
use super::ReplyPlan;
use super::probe::{request, send, service};
use std::net::{IpAddr, Ipv4Addr};

pub fn pinned() {
    let svc = service(ReplyPlan::status(201));
    let req = request(svc.addr.ip(), 443);
    assert_eq!(send(&svc, &req, 2_000).unwrap().status, 201);
    let mut wrong = request(svc.addr.ip(), 443);
    wrong.ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2));
    assert!(send(&svc, &wrong, 500).is_err());
}
