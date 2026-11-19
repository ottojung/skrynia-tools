use super::core::DurableBackend;
use super::data::{RandomSource, StateStore};
use super::{CompatError, CompatResult, Mutation, MutationKind};
use sha2::Digest;

pub fn mutate<S: StateStore, R: RandomSource>(
    backend: &DurableBackend<S, R>,
    req: Mutation<'_>,
) -> CompatResult<bool> {
    let mut state = backend.lock();
    let authority = DurableBackend::<S, R>::authority(&state)?;
    let operation = match req.kind {
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
    let grant = authority.grant_for(req.token, operation, req.key)?;
    if let Some(expected) = req.expected {
        let old = state
            .objects
            .get(req.key)
            .ok_or_else(|| CompatError::new(404, "not_found"))?;
        if hex::encode(sha2::Sha256::digest(old)) != expected {
            return Err(CompatError::new(412, "content_mismatch"));
        }
    }
    if req.kind != MutationKind::Delete {
        let sizes = state
            .objects
            .iter()
            .map(|(k, v)| (k.clone(), v.len() as u64))
            .collect();
        authority.check_write_quota(&grant.id, req.key, &sizes, req.body.len() as u64, &[])?;
        state.objects.insert(req.key.into(), req.body.to_vec());
    } else {
        state.objects.remove(req.key);
    }
    backend.persist(&state)?;
    Ok(false)
}
