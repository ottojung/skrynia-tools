use super::*;
pub(super) fn root() -> Grant {
    Grant {
        id: "root".into(),
        parent: None,
        locations: "team/*".into(),
        operations: vec!["create".into(), "delegate".into()],
        max_bytes: None,
        max_objects: None,
        issuer_hash: "issuer".into(),
        root_issuer_token: None,
        users: vec![],
    }
}
pub(super) fn child() -> Grant {
    Grant {
        id: "child".into(),
        parent: Some("root".into()),
        locations: "team/a*".into(),
        operations: vec!["create".into()],
        max_bytes: Some(4000),
        max_objects: Some(5),
        issuer_hash: "child-issuer".into(),
        root_issuer_token: None,
        users: vec![],
    }
}
