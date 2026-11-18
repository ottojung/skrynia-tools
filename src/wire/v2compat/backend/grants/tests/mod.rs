use super::*;
use sha2::{Digest, Sha256};
fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token))
}
fn authority() -> Authority {
    Authority::from_grants([Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into()],
        max_bytes: None,
        max_objects: None,
        issuer_hash: hash("issuer"),
        root_issuer_token: Some("issuer".into()),
        users: vec![User {
            id: "user".into(),
            hash: hash("token"),
        }],
    }])
    .unwrap()
}
#[test]
fn v2_grant_checks_are_exact() {
    let auth = authority();
    assert!(auth.grant_for("token", "create", "team/a").is_ok());
    for (token, op, key, expected) in [
        ("bad", "create", "team/a", "invalid_grant_user_token"),
        ("token", "list", "team/a", "grant_operation_forbidden"),
        ("token", "create", "other/a", "location_not_authorized"),
    ] {
        assert_eq!(auth.grant_for(token, op, key).unwrap_err().code, expected);
    }
}
mod fixtures;
mod issuer;
mod validate1;
mod validate2;
use fixtures::{child, root};
mod child_spec;
mod child_spec_invalid;
use child_spec::valid;
