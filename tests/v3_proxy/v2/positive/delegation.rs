use super::{call, fixture};
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Grant, User};
#[test]
#[ignore = "requires actual token-gated Program0, generic lock kernel, ordinary v2 guest"]
fn delegated_child_authority_obeys_scope_operation_and_ancestor_quota() {
    let (s, _) = fixture::start(Some(5000));
    let token = "b".repeat(64);
    let child = Grant {
        id: "child".into(),
        parent: Some("root".into()),
        locations: "team/c*".into(),
        operations: vec!["create".into()],
        max_bytes: Some(5000),
        max_objects: None,
        issuer_hash: hex::encode(Sha256::digest(b"child_issuer")),
        root_issuer_token: None,
        users: vec![User {
            id: "delegated".into(),
            hash: hex::encode(Sha256::digest(token.as_bytes())),
        }],
    };
    s.seed_file(
        "v2/state/grants/child.json",
        &serde_json::to_vec(&child).unwrap(),
    );
    let create = fixture::req("POST", "team/c1", &token, b"hi");
    assert_eq!(call::call(&s, &create).status, 201);
    let forbidden = fixture::req("POST", "team/d1", &token, b"hi");
    assert_eq!(call::call(&s, &forbidden).status, 403);
    let forbidden = fixture::req("PUT", "team/c1", &token, b"updated");
    assert_eq!(call::call(&s, &forbidden).status, 403);
    let quota = fixture::req("POST", "team/c2", &token, b"hi");
    assert_eq!(call::call(&s, &quota).status, 507);
    assert_eq!(s.read_file("v2/objects/team/c1").unwrap(), b"hi");
    assert!(s.read_file("v2/objects/team/c2").is_none());
}
