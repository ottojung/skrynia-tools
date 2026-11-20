use super::{grants, lock, objects, scan};
use crate::bridge::Callback;
use sha2::{Digest, Sha256};
use skrynia_tools::wire::v2compat::{CompatError, Mutation, MutationKind};

fn err(status: u16, code: &str) -> CompatError {
    CompatError::new(status, code)
}
pub fn mutate<I: Callback>(i: &I, req: Mutation<'_>) -> Result<bool, CompatError> {
    let guard = lock::acquire(i).ok_or_else(|| err(503, "unavailable"))?;
    let result = mutate_locked(i, req);
    if lock::release(i, guard).is_err() {
        return Err(err(503, "mutation_outcome_unknown"));
    }
    result
}
fn mutate_locked<I: Callback>(i: &I, req: Mutation<'_>) -> Result<bool, CompatError> {
    let op = match req.kind {
        MutationKind::Create => "create",
        MutationKind::Replace => "modify",
        MutationKind::Delete => "delete",
    };
    let current = objects::read(i, req.key).map_err(|_| err(503, "unavailable"))?;
    if req.kind == MutationKind::Create && current.is_some() {
        return Err(err(409, "already_exists"));
    }
    if req.kind != MutationKind::Create && current.is_none() {
        return Err(err(404, "not_found"));
    }
    if let Some(expected) = req.expected {
        let old = current
            .as_ref()
            .ok_or_else(|| err(412, "content_mismatch"))?;
        if hex::encode(Sha256::digest(old)) != expected {
            return Err(err(412, "content_mismatch"));
        }
    }
    let auth = grants::load(i).map_err(|_| err(403, "invalid_grant_user_token"))?;
    let grant = auth.grant_for(req.token, op, req.key)?;
    if req.kind != MutationKind::Delete {
        let sizes = scan::object_sizes(i).map_err(|_| err(503, "unavailable"))?;
        auth.check_write_quota(&grant.id, req.key, &sizes, req.body.len() as u64, &[])?;
    }
    match req.kind {
        MutationKind::Delete => objects::remove(i, req.key).map_err(|_| err(503, "unavailable"))?,
        _ => objects::write(i, req.key, req.body).map_err(|_| err(503, "unavailable"))?,
    }
    Ok(false)
}
