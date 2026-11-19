use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::{CompatError, CompatResult, User};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

fn hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}

impl<S: StateStore, R: RandomSource> DurableBackend<S, R> {
    pub fn issue_user(&self, grant_id: &str, issuer: &str) -> CompatResult<Value> {
        self.change(|state| {
            let authority = DurableBackend::<S, R>::authority(&state)?;
            authority.issuer(grant_id, issuer)?;
            let users = state
                .grants
                .iter()
                .find(|g| g.id == grant_id)
                .ok_or_else(|| CompatError::new(404, "grant_not_found"))?
                .users
                .len();
            super::quota::check_user_add(&authority, grant_id, users, &state)?;
            let user_id = self.random_hex(16)?;
            if state
                .grants
                .iter()
                .any(|grant| grant.users.iter().any(|u| u.id == user_id))
            {
                return Err(CompatError::new(500, "duplicate_user_id"));
            }
            let token = self.random_hex(32)?;
            let grant = state
                .grants
                .iter_mut()
                .find(|g| g.id == grant_id)
                .ok_or_else(|| CompatError::new(404, "grant_not_found"))?;
            grant.users.push(User {
                id: user_id.clone(),
                hash: hash(&token),
            });
            Ok(json!({"grantId": grant_id, "userId": user_id, "userToken": token}))
        })
    }
}
