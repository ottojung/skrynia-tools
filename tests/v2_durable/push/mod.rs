mod persist;
pub(super) use super::fakes::{M, R};
use serde_json::{Value, json};
use skrynia_tools::wire::v2compat::Request;
use std::collections::BTreeMap;

pub(super) fn req(
    method: &str,
    path: &str,
    headers: BTreeMap<String, String>,
    body: Value,
) -> Request {
    Request {
        method: method.into(),
        path: path.into(),
        query: None,
        headers,
        body: serde_json::to_vec(&body).unwrap(),
    }
}
pub(super) fn auth(token: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("authorization".into(), format!("Bearer {token}"))])
}
pub(super) fn cap(token: &str) -> BTreeMap<String, String> {
    BTreeMap::from([("x-skrynia-capability".into(), token.into())])
}
pub(super) fn sub_body(loc: &str) -> Value {
    json!({"locations": loc, "endpoint": "https://push.example/1", "keys": {"p256dh": "k", "auth": "a"}})
}
