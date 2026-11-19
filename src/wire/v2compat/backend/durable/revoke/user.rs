use super::*;
impl<S: StateStore, R> DurableBackend<S, R> {
    pub fn revoke_user(&self, grant_id: &str, user_id: &str, issuer: &str) -> CompatResult<()> {
        self.change(|state| {
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
            Ok(())
        })
    }
}
