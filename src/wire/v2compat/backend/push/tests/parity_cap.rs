use super::parity_sup::{G, I, sreq};
use super::{Mem, Request, api};
use std::collections::BTreeMap;
#[test]
fn capability_required_for_update_delete() {
    let mut s = Mem::default();
    let body = serde_json::json!({"locations":"apps/x/*","endpoint":"https://p/1","keys":{"p256dh":"k","auth":"a"}});
    let r = api::subscribe(&mut s, &G, &I(2), &sreq(&"b".repeat(64), body));
    let v: serde_json::Value = serde_json::from_slice(&r.body).unwrap();
    let id = v["id"].as_str().unwrap();
    let up = serde_json::json!({"endpoint":"https://p/2","keys":{"p256dh":"k","auth":"a"}});
    let mut h = BTreeMap::new();
    h.insert("x-skrynia-capability".into(), format!("{:064x}", 3u32));
    let req = Request {
        method: "PUT".into(),
        path: format!("/push/subscriptions/{id}"),
        query: None,
        headers: h,
        body: serde_json::to_vec(&up).unwrap(),
    };
    assert_eq!(api::update(&mut s, id, &req).status, 403);
}
