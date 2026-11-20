use super::{call, fixture, parent_fixture::Server};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};

#[test]
#[ignore = "needs real v2 WASM, grant-aware generic callback and lock kernel"]
fn issued_grants_and_users_survive_restart() {
    let mut s = Server::start(Some(&fixture::guest()));
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
        br#"{"locations":"team/b/*","operations":["create"],"maxBytes":4096,"maxObjects":5}"#
            .to_vec();
    let v: serde_json::Value = serde_json::from_slice(&call::call(&s, &req).body).unwrap();
    let (id, iss) = (
        v["id"].as_str().unwrap().to_string(),
        v["issuerToken"].as_str().unwrap().to_string(),
    );
    let mut issue = call::req("POST", &format!("/grants/{id}/users"));
    issue
        .headers
        .insert("authorization".into(), format!("Bearer {iss}"));
    issue.body = b"{}".to_vec();
    let tok = serde_json::from_slice::<serde_json::Value>(&call::call(&s, &issue).body).unwrap()
        ["userToken"].as_str().unwrap().to_string();
    s.restart_with_guest(&fixture::guest());
    let mut get = call::req("GET", &format!("/grants/{id}"));
    get.headers
        .insert("authorization".into(), format!("Bearer {tok}"));
    assert_eq!(call::call(&s, &get).status, 200);
}
