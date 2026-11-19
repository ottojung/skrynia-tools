use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::{CompatError, CompatResult, Grant};
use sha2::{Digest, Sha256};

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

impl<S: StateStore, R: RandomSource> DurableBackend<S, R> {
    pub fn bootstrap_root(&self, id: &str, issuer_token: &str) -> CompatResult<()> {
        self.change(|state| {
            if !state.grants.is_empty() {
                return Err(CompatError::new(409, "already_exists"));
            }
            state.grants.push(Grant {
                id: id.into(),
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
                issuer_hash: hash(issuer_token),
                root_issuer_token: None,
                users: Vec::new(),
            });
            Ok(())
        })
    }
}
