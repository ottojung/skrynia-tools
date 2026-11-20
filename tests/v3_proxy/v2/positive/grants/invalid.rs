use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

fn seed() -> (Server, String, String) {
    let s = Server::start(Some(&fixture::guest()));
    let issuer = "1".repeat(64);
    let user = "2".repeat(64);
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
    (s, user, issuer)
}

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn invalid_issuer_and_widened_scope_are_rejected() {
    let (s, user, _issuer) = seed();
    let mut bad_issue = call::req("POST", "/grants/root/users");
    bad_issue
        .headers
        .insert("authorization".into(), format!("Bearer {}", "3".repeat(64)));
    bad_issue.body = b"{}".to_vec();
    assert_eq!(call::call(&s, &bad_issue).status, 403);
    let mut wide = call::req("POST", "/grants");
    wide.headers
        .insert("authorization".into(), format!("Bearer {user}"));
    wide.body =
        br#"{"locations":"other/*","operations":["create"],"maxBytes":4096,"maxObjects":1}"#
            .to_vec();
    let r = call::call(&s, &wide);
    assert_eq!(r.status, 403);
    let body = String::from_utf8_lossy(&r.body);
    assert!(body.contains("locations_outside_parent"));
}
