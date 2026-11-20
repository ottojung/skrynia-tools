use super::{call, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{
    Request,
    grants::{Grant, User},
};
pub fn guest() -> Vec<u8> {
    std::fs::read(std::env::var("SKRYNIA_V2_PROVIDER_WASM").unwrap()).unwrap()
}
pub fn start(limit: Option<u64>) -> (Server, String) {
    let s = Server::start(Some(&guest()));
    let token = "a".repeat(64);
    let grant = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "modify".into(), "delete".into()],
        max_bytes: limit,
        max_objects: None,
        issuer_hash: hex::encode(Sha256::digest(b"issuer")),
        root_issuer_token: None,
        users: vec![User {
            id: "user1".into(),
            hash: hex::encode(Sha256::digest(token.as_bytes())),
        }],
    };
    s.seed_file(
        "v2/state/grants/root.json",
        &serde_json::to_vec(&grant).unwrap(),
    );
    (s, token)
}
pub fn req(method: &str, key: &str, token: &str, body: &[u8]) -> Request {
    let mut r = call::req(method, &format!("/store/{key}"));
    r.headers
        .insert("authorization".into(), format!("Bearer {token}"));
    r.body = body.to_vec();
    r
}
