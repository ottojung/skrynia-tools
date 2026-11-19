use super::{M, R};
use skrynia_tools::wire::v2compat::{DurableBackend, Request, handle};
use std::sync::atomic::AtomicU64;
pub type B = DurableBackend<M, R>;
pub fn setup() -> (B, String, String) {
    let b = B::open(M::default(), R(AtomicU64::new(1))).unwrap();
    let issuer = "a".repeat(64);
    b.bootstrap_root("r", &issuer).unwrap();
    let u = b.issue_user("r", &issuer).unwrap();
    (b, issuer, u["userToken"].as_str().unwrap().into())
}
pub fn call(b: &B, method: &str, path: &str, token: &str, body: &[u8]) -> (u16, serde_json::Value) {
    let mut req = Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers: Default::default(),
        body: body.to_vec(),
    };
    if !token.is_empty() {
        req.headers
            .insert("Authorization".into(), format!("Bearer {token}"));
    }
    let r = handle(b, &req);
    (r.status, serde_json::from_slice(&r.body).unwrap())
}
pub fn child(b: &B, parent: &str) -> serde_json::Value {
    let spec = serde_json::json!({"locations":"t/*",
        "operations":["create","modify","delete"],"maxBytes":4096,"maxObjects":2});
    let (status, reply) = call(
        b,
        "POST",
        "/grants",
        parent,
        &serde_json::to_vec(&spec).unwrap(),
    );
    assert_eq!(status, 200);
    assert_eq!(reply["locations"], "t/*");
    assert_eq!(reply["operations"], spec["operations"]);
    assert_eq!(reply["maxBytes"], 4096);
    reply
}
