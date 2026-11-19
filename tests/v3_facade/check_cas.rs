use super::http::{B, call, digest, json};
use serde_json::json;

fn put(b: &B, key: &str, body: Vec<u8>, t: &str, cas: &str) -> u16 {
    let h = vec![("X-Skrynia-If-SHA256", cas)];
    call(b, "PUT", &format!("/store/{key}"), None, t, body, h).status
}
pub fn rollback(b: &B, t: &str, vault: &[u8]) {
    let old = call(
        b,
        "GET",
        "/store/apps/alpha/active.json",
        None,
        "",
        vec![],
        vec![],
    );
    assert_eq!(old.status, 200);
    let nb = json(&json!({"schema":1,"release":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","nonce":"n2"}));
    assert_eq!(
        put(b, "apps/alpha/active.json", nb, t, &digest(&old.body)),
        200
    );
    let cur = call(
        b,
        "GET",
        "/store/apps/alpha/active.json",
        None,
        "",
        vec![],
        vec![],
    );
    let c: serde_json::Value = serde_json::from_slice(&cur.body).unwrap();
    assert_eq!(c["release"], "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let vr = call(
        b,
        "GET",
        "/store/apps/tools/vault/00112233445566778899aabbccddeeff.json",
        None,
        "",
        vec![],
        vec![],
    );
    assert_eq!(vr.body, vault);
    let stale =
        json(&json!({"schema":1,"release":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","nonce":"n3"}));
    assert_eq!(
        put(b, "apps/alpha/active.json", stale, t, &digest(&old.body)),
        412
    );
}
