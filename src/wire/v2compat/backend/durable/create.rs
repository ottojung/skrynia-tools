use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::{CompatResult, Grant};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

impl<S: StateStore, R: RandomSource> DurableBackend<S, R> {
    pub fn create_grant(&self, parent_token: &str, spec: &Value) -> CompatResult<Value> {
        let mut state = self.lock();
        let authority = DurableBackend::<S, R>::authority(&state)?;
        let valid = authority.child_spec(parent_token, spec)?;
        let parent_id = authority.user(parent_token)?.id.clone();
        super::quota::check_meta_add(&authority, &parent_id, &state)?;
        state.next_grant = state.next_grant.saturating_add(1);
        let id = format!("g{}", state.next_grant);
        let issuer = self.random_hex(32)?;
        state.grants.push(Grant {
            id: id.clone(),
            parent: Some(parent_id),
            locations: valid.locations,
            operations: valid.operations,
            max_bytes: Some(valid.max_bytes),
            max_objects: Some(valid.max_objects),
            issuer_hash: hash(&issuer),
            root_issuer_token: None,
            users: Vec::new(),
        });
        self.persist(&state)?;
        Ok(json!({"id": id, "issuerToken": issuer}))
    }
}
