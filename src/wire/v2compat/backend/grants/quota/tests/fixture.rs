use super::{Authority, Grant, User};
use std::collections::HashMap;
pub fn fixture() -> (Authority, HashMap<String, u64>) {
    let root = Grant {
        id: "root".into(),
        parent: None,
        locations: "*".into(),
        operations: vec!["create".into(), "delegate".into()],
        max_bytes: None,
        max_objects: None,
        issuer_hash: "x".into(),
        root_issuer_token: None,
        users: vec![],
    };
    let child = Grant {
        id: "child".into(),
        parent: Some("root".into()),
        locations: "team/*".into(),
        operations: vec!["create".into()],
        max_bytes: Some(3000),
        max_objects: Some(1),
        issuer_hash: "y".into(),
        root_issuer_token: None,
        users: vec![
            User {
                id: "one".into(),
                hash: "a".into(),
            },
            User {
                id: "two".into(),
                hash: "b".into(),
            },
        ],
    };
    let mut sizes = HashMap::new();
    sizes.insert("outsider".into(), 100);
    (Authority::from_grants([root, child]).unwrap(), sizes)
}
