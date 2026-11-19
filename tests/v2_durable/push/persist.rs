use super::{M, R, auth, cap, req, sub_body};
use serde_json::{Value, json};
use skrynia_tools::wire::v2compat::{DurableBackend, handle};
use std::sync::atomic::AtomicU64;

#[test]
fn durable_subscriptions_persist_and_authorize() {
    let store = M::default();
    let b = DurableBackend::open(store.clone(), R(AtomicU64::new(1))).unwrap();
    b.bootstrap_root("r", "issuer").unwrap();
    let root_user = b.issue_user("r", "issuer").unwrap()["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    let g = b
        .create_grant(
            &root_user,
            &json!({"locations":"apps/*","operations":["subscribe"],"maxBytes":8192,"maxObjects":2}),
        )
        .unwrap();
    let gid = g["id"].as_str().unwrap();
    let giss = g["issuerToken"].as_str().unwrap();
    let ut = b.issue_user(gid, giss).unwrap()["userToken"]
        .as_str()
        .unwrap()
        .to_string();
    let r = handle(
        &b,
        &req(
            "POST",
            "/push/subscriptions",
            auth(&ut),
            sub_body("apps/a/*"),
        ),
    );
    assert_eq!(r.status, 201, "{}", String::from_utf8_lossy(&r.body));
    let v: Value = serde_json::from_slice(&r.body).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    let capability = v["capability"].as_str().unwrap().to_string();
    drop(b);
    let b = DurableBackend::open(store, R(AtomicU64::new(100))).unwrap();
    let up = json!({"endpoint":"https://push.example/2","keys":{"p256dh":"k2","auth":"a2"}});
    let path = format!("/push/subscriptions/{id}");
    let r = handle(&b, &req("PUT", &path, cap(&capability), up));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let r = handle(&b, &req("DELETE", &path, cap(&capability), json!({})));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
}
