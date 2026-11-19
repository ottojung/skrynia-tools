use super::core::DurableBackend;
use super::data::StateStore;
use super::{CompatError, CompatResult};

impl<S: StateStore, R> DurableBackend<S, R> {
    pub fn revoke_user(&self, grant_id: &str, user_id: &str, issuer: &str) -> CompatResult<()> {
        let mut state = self.lock();
        let authority = DurableBackend::<S, R>::authority(&state)?;
        authority.issuer(grant_id, issuer)?;
        let grant = state
            .grants
            .iter_mut()
            .find(|g| g.id == grant_id)
            .ok_or_else(|| CompatError::new(404, "grant_not_found"))?;
        let before = grant.users.len();
        grant.users.retain(|u| u.id != user_id);
        if grant.users.len() == before {
            return Err(CompatError::new(404, "user_not_found"));
        }
        self.persist(&state)
    }
    pub fn revoke_grant(&self, grant_id: &str, issuer: &str) -> CompatResult<()> {
        let mut state = self.lock();
        let authority = DurableBackend::<S, R>::authority(&state)?;
        authority.issuer(grant_id, issuer)?;
        let mut removed = std::collections::HashSet::new();
        removed.insert(grant_id.to_string());
        loop {
            let more: Vec<String> = state
                .grants
                .iter()
                .filter(|g| {
                    g.parent
                        .as_deref()
                        .is_some_and(|p| removed.contains(p) && !removed.contains(&g.id))
                })
                .map(|g| g.id.clone())
                .collect();
            if more.is_empty() {
                break;
            }
            removed.extend(more);
        }
        if !state.grants.iter().any(|g| g.id == grant_id) {
            return Err(CompatError::new(404, "grant_not_found"));
        }
        state.grants.retain(|g| !removed.contains(&g.id));
        self.persist(&state)
    }
}
