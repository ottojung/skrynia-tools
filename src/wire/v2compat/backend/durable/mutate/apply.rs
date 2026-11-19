use super::{
    CompatError, CompatResult, DurableBackend, Mutation, MutationKind, RandomSource, StateStore,
    write,
};
use sha2::Digest;

pub fn run<S: StateStore, R: RandomSource>(
    backend: &DurableBackend<S, R>,
    req: Mutation<'_>,
) -> CompatResult<bool> {
    backend.change(|state| {
        let authority = DurableBackend::<S, R>::authority(state)?;
        let op = match req.kind {
            MutationKind::Create => "create",
            MutationKind::Replace => "modify",
            MutationKind::Delete => "delete",
        };
        if req.kind == MutationKind::Create && state.objects.contains_key(req.key) {
            return Err(CompatError::new(409, "already_exists"));
        }
        if req.kind != MutationKind::Create && !state.objects.contains_key(req.key) {
            return Err(CompatError::new(404, "not_found"));
        }
        let grant = authority.grant_for(req.token, op, req.key)?;
        if let Some(expected) = req.expected {
            let old = state
                .objects
                .get(req.key)
                .ok_or_else(|| CompatError::new(404, "not_found"))?;
            if hex::encode(sha2::Sha256::digest(old)) != expected {
                return Err(CompatError::new(412, "content_mismatch"));
            }
        }
        write::apply(state, &authority, &grant.id, &req)?;
        Ok(false)
    })
}
