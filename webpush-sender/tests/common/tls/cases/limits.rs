//! Retry-After cap, overall timeout and bounded response body.
use super::ReplyPlan;
use super::probe::{request, send, service};

pub fn limits() {
    let limited = service(ReplyPlan::status(429).retry(1_000_000));
    let req = request(limited.addr.ip(), 443);
    let reply = send(&limited, &req, 2_000).unwrap();
    assert_eq!((reply.status, reply.retry_after_ms), (429, Some(60_000)));
    let slow = service(ReplyPlan::slow(800));
    let req = request(slow.addr.ip(), 443);
    assert!(send(&slow, &req, 120).is_err());
    let mut huge = ReplyPlan::status(201);
    huge.body = vec![b'x'; 70 * 1024];
    let svc = service(huge);
    let req = request(svc.addr.ip(), 443);
    assert!(send(&svc, &req, 2_000).is_err());
}
