use super::support::{G, I, req};
use super::{Mem, Store, api};
#[test]
fn subscribe_update_delete_roundtrip() {
    let mut s = Mem::default();
    let g = G("a".repeat(64), "apps/*".into());
    let b = serde_json::json!({"locations":"apps/x/*","endpoint":"https://push.example/1","keys":{"p256dh":"k","auth":"a"}});
    let r = api::subscribe(&mut s, &g, &I(1), &req(Some(&"a".repeat(64)), None, b));
    assert_eq!(r.status, 201);
    let v: serde_json::Value = serde_json::from_slice(&r.body).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    let cap = v["capability"].as_str().unwrap().to_string();
    let up =
        serde_json::json!({"endpoint":"https://push.example/2","keys":{"p256dh":"k2","auth":"a2"}});
    let mut ur = req(None, Some(&cap), up);
    ur.path = format!("/push/subscriptions/{id}");
    assert_eq!(api::update(&mut s, &id, &ur).status, 200);
    let mut dr = req(None, Some(&cap), serde_json::json!({}));
    dr.body = b"{}".to_vec();
    assert_eq!(api::delete(&mut s, &id, &dr).status, 200);
    assert!(s.get(&id).is_none());
}
