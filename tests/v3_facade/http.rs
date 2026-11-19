use super::fake::{M, R};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{DurableBackend, Request, handle};

pub type B = DurableBackend<M, R>;
pub fn call(
    b: &B,
    method: &str,
    path: &str,
    query: Option<&str>,
    token: &str,
    body: Vec<u8>,
    headers: Vec<(&str, &str)>,
) -> skrynia_tools::wire::v2compat::Response {
    let mut req = Request {
        method: method.into(),
        path: path.into(),
        query: query.map(str::to_owned),
        headers: Default::default(),
        body,
    };
    if !token.is_empty() {
        req.headers
            .insert("Authorization".into(), format!("Bearer {token}"));
    }
    for (k, v) in headers {
        req.headers.insert(k.into(), v.into());
    }
    handle(b, &req)
}
pub fn digest(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
pub fn json(body: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(body).unwrap()
}
