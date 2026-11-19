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
        let mut state = self.lock();
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
        state.next_user = state.next_user.saturating_add(1);
        let user_id = format!("u{}", state.next_user);
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
        self.persist(&state)?;
        Ok(json!({"grantId": grant_id, "userId": user_id, "userToken": token}))
    }
}
