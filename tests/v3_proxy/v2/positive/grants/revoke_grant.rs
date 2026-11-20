use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn revoked_grant_subtree_is_removed() {
    let s = Server::start(Some(&fixture::guest()));
    let (issuer, user) = ("1".repeat(64), "2".repeat(64));
    let g = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into()],
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
    let mut req = call::req("POST", "/grants");
    req.headers
        .insert("authorization".into(), format!("Bearer {user}"));
    req.body =
        br#"{"locations":"team/c/*","operations":["create"],"maxBytes":4096,"maxObjects":2}"#
            .to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &req).body).unwrap();
    let (id, child_iss) = (
        v["id"].as_str().unwrap(),
        v["issuerToken"].as_str().unwrap(),
    );
    let mut del = call::req("DELETE", &format!("/grants/{id}"));
    del.headers
        .insert("authorization".into(), format!("Bearer {child_iss}"));
    assert_eq!(call::call(&s, &del).status, 200);
    assert!(s.read_file(&format!("v2/state/grants/{id}.json")).is_none());
}
