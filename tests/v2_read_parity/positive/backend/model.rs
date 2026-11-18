use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Authority, Grant, User};
use std::{collections::HashMap, sync::Mutex};
pub struct RootStore {
    pub(super) authority: Authority,
    pub(super) data: Mutex<HashMap<String, Vec<u8>>>,
}
impl RootStore {
    pub fn new(token: String) -> Self {
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
            issuer_hash: hex::encode(Sha256::digest(b"fixture-issuer")),
            root_issuer_token: None,
            users: vec![User {
                id: "user".into(),
                hash: hex::encode(Sha256::digest(token.as_bytes())),
            }],
        };
        Self {
            authority: Authority::from_grants([root]).unwrap(),
            data: Mutex::new(HashMap::new()),
        }
    }
}
