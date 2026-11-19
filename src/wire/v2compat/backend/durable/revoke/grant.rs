use super::*;
impl<S: StateStore, R> DurableBackend<S, R> {
    pub fn revoke_grant(&self, grant_id: &str, issuer: &str) -> CompatResult<()> {
        self.change(|state| {
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
            Ok(())
        })
    }
}
