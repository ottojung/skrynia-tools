use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub hash: String,
    pub ops: Vec<String>,
    pub locations: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Authority {
    pub users: Vec<User>,
}
impl Authority {
    pub fn root(hash: String) -> Self {
        Self {
            users: vec![User {
                hash,
                ops: vec!["create".into(), "modify".into(), "delete".into()],
                locations: "*".into(),
            }],
        }
    }
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
    pub fn allows(&self, token_hash: &str, op: &str, key: &str) -> bool {
        self.users.iter().any(|u| {
            u.hash == token_hash
                && u.ops.iter().any(|o| o == op)
                && matches_scope(&u.locations, key)
        })
    }
}
fn matches_scope(scope: &str, key: &str) -> bool {
    if scope == "*" {
        return true;
    }
    if let Some(p) = scope.strip_suffix("/*") {
        return key.starts_with(&format!("{p}/"));
    }
    if let Some(p) = scope.strip_suffix('*') {
        return key.starts_with(p);
    }
    scope == key
}
