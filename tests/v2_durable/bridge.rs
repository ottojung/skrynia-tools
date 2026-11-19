use super::fakes::{M, R};
use skrynia_tools::wire::v2compat::{DurableBackend, Request, handle};
use std::sync::atomic::AtomicU64;
#[test]
fn durable_push_subscribe_requires_grant_user_token() {
    let authority = DurableBackend::open(M::default(), R(AtomicU64::new(1))).unwrap();
    authority.bootstrap_root("root", "secret").unwrap();
    let req = Request {
        method: "POST".into(),
        path: "/push/subscriptions".into(),
        query: None,
        headers: Default::default(),
        body: b"{}".to_vec(),
    };
    let reply = handle(&authority, &req);
    assert_eq!(reply.status, 401);
    assert_eq!(reply.body, br#"{"error":"grant_user_token_required"}"#);
}
