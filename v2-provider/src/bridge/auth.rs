use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::grants::{Authority, Grant, User};
pub const TEST_TOKEN: &str = "0000000000000000000000000000000000000000000000000000000000000000";
pub fn authority() -> Authority {
    let hash = hex::encode(Sha256::digest(TEST_TOKEN.as_bytes()));
    Authority::from_grants([Grant {
        id: "root".into(),
        parent: None,
        locations: "test/*".into(),
        operations: ["create", "modify", "delete", "list"]
            .map(str::to_string)
            .into(),
        max_bytes: None,
        max_objects: None,
        issuer_hash: hash.clone(),
        root_issuer_token: None,
        users: vec![User {
            id: "user".into(),
            hash,
        }],
    }])
    .unwrap()
}
