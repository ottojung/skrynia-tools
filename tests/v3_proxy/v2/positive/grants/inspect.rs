use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn exact_user_can_inspect_grant_and_usage() {
    let s = Server::start(Some(&fixture::guest()));
    let (issuer, user) = ("1".repeat(64), "2".repeat(64));
    let g = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "list".into()],
        max_bytes: None,
        max_objects: None,
        issuer_hash: hex::encode(Sha256::digest(issuer.as_bytes())),
        root_issuer_token: None,
        users: vec![User {
            id: "u1".into(),
            hash: hex::encode(Sha256::digest(user.as_bytes())),
        }],
    };
    s.seed_file(
        "v2/state/grants/root.json",
        &serde_json::to_vec(&g).unwrap(),
    );
    let mut req = call::req("GET", "/grants/root");
    req.headers
        .insert("authorization".into(), format!("Bearer {user}"));
    let resp = call::call(&s, &req);
    assert_eq!(resp.status, 200);
    let v: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
    assert_eq!(v["id"], "root");
    assert_eq!(v["locations"], "team/*");
    assert!(v.get("bytesUsed").is_some());
    assert!(v.get("objectsUsed").is_some());
}
