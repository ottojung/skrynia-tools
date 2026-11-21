use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn creates_narrow_child_and_issues_user_token() {
    let s = Server::start(Some(&fixture::guest()));
    let (issuer, user) = ("1".repeat(64), "2".repeat(64));
    let grant = Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into(), "list".into()],
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
        &serde_json::to_vec(&grant).unwrap(),
    );
    let mut req = call::req("POST", "/grants");
    req.headers
        .insert("authorization".into(), format!("Bearer {user}"));
    req.body =
        br#"{"locations":"team/a/*","operations":["create"],"maxBytes":4096,"maxObjects":10}"#
            .to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &req).body).unwrap();
    let (id, iss) = (
        v["id"].as_str().unwrap(),
        v["issuerToken"].as_str().unwrap(),
    );
    let mut issue = call::req("POST", &format!("/grants/{id}/users"));
    issue
        .headers
        .insert("authorization".into(), format!("Bearer {iss}"));
    issue.body = b"{}".to_vec();
    let u: serde_json::Value = serde_json::from_slice(&call::call(&s, &issue).body).unwrap();
    assert_eq!(u["grantId"], id);
    assert!(!u["userToken"].as_str().unwrap().is_empty());
}
