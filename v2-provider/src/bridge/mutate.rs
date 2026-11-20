use super::{Callback, InvokeBackend, authority, object, wire};
use skrynia_tools::wire::v2compat::{
    CompatError, CompatResult, Mutation, MutationKind, ObjectRead, Store,
};
use std::collections::HashMap;
pub(super) fn apply<I: Callback>(
    backend: &InvokeBackend<'_, I>,
    req: Mutation<'_>,
) -> CompatResult<bool> {
    let op = match req.kind {
        MutationKind::Create => "create",
        MutationKind::Replace => "modify",
        MutationKind::Delete => "delete",
    };
    let auth = authority();
    let grant = auth.grant_for(req.token, op, req.key)?;
    let old = match backend.read_object(req.key)? {
        ObjectRead::Missing => None,
        ObjectRead::Found(o) => Some(o.bytes),
    };
    if req.kind == MutationKind::Create && old.is_some() {
        return Err(CompatError::new(409, "already_exists"));
    }
    if req.kind != MutationKind::Create && old.is_none() {
        return Err(CompatError::new(404, "not_found"));
    }
    if let Some(expected) = req.expected {
        use sha2::Digest;
        let actual = hex::encode(sha2::Sha256::digest(old.as_deref().unwrap_or(&[])));
        if actual != expected {
            return Err(CompatError::new(412, "content_mismatch"));
        }
    }
    if req.kind != MutationKind::Delete {
        let mut sizes = HashMap::new();
        if let Some(bytes) = &old {
            sizes.insert(req.key.to_string(), bytes.len() as u64);
        }
        auth.check_write_quota(&grant.id, req.key, &sizes, req.body.len() as u64, &[])?;
    }
    commit(backend, req.kind, req.key, req.body)?;
    Ok(false)
}
fn commit<I: Callback>(
    b: &InvokeBackend<'_, I>,
    kind: MutationKind,
    key: &str,
    body: &[u8],
) -> CompatResult<()> {
    let req = match kind {
        MutationKind::Delete => wire::key_req(3, key),
        _ => wire::key_body_req(2, key, body),
    }
    .ok_or_else(object::unavailable)?;
    wire::ok(&b.invoke.invoke(&req).map_err(|_| object::unavailable())?)
        .map_err(|_| object::unavailable())
}
