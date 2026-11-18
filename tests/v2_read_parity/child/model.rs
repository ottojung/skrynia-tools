use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Authority, Grant, User};
pub fn limited(token: &str) -> Authority {
    let root = Grant {
        id: "root".into(),
        parent: None,
        locations: "*".into(),
        operations: [
            "create",
            "modify",
            "delete",
            "list",
            "delegate",
            "subscribe",
        ]
        .map(str::to_string)
        .into(),
        max_bytes: None,
        max_objects: None,
        issuer_hash: "fixture".into(),
        root_issuer_token: None,
        users: vec![],
    };
    let child = Grant {
        id: "child".into(),
        parent: Some("root".into()),
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into()],
        max_bytes: Some(10000),
        max_objects: Some(10),
        issuer_hash: "fixture-child".into(),
        root_issuer_token: None,
        users: vec![User {
            id: "child-user".into(),
            hash: hex::encode(Sha256::digest(token.as_bytes())),
        }],
    };
    Authority::from_grants([root, child]).unwrap()
}
