use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn revoked_user_token_cannot_be_used() {
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
    let mut issue = call::req("POST", "/grants/root/users");
    issue
        .headers
        .insert("authorization".into(), format!("Bearer {issuer}"));
    issue.body = b"{}".to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &issue).body).unwrap();
    let (uid, tok) = (
        v["userId"].as_str().unwrap(),
        v["userToken"].as_str().unwrap(),
    );
    let mut del = call::req("DELETE", &format!("/grants/root/users/{uid}"));
    del.headers
        .insert("authorization".into(), format!("Bearer {issuer}"));
    assert_eq!(call::call(&s, &del).status, 200);
    let mut inspect = call::req("GET", "/grants/root");
    inspect
        .headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    assert_eq!(call::call(&s, &inspect).status, 403);
}
