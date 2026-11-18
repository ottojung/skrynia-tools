use super::*;
pub fn apply(store: &RootStore, req: Mutation<'_>) -> CompatResult<bool> {
    let fail = |status, code| CompatError::new(status, code);
    let operation = match req.kind {
        MutationKind::Create => "create",
        MutationKind::Replace => "modify",
        MutationKind::Delete => "delete",
    };
    let mut map = store.data.lock().unwrap();
    let old = map.get(req.key);
    if req.kind == MutationKind::Create && old.is_some() {
        return Err(fail(409, "already_exists"));
    }
    if req.kind != MutationKind::Create && old.is_none() {
        return Err(fail(404, "not_found"));
    }
    let grant = store.authority.grant_for(req.token, operation, req.key)?;
    if let Some(expected) = req.expected {
        use sha2::Digest;
        if hex::encode(sha2::Sha256::digest(old.unwrap())) != expected {
            return Err(fail(412, "content_mismatch"));
        }
    }
    if req.kind != MutationKind::Delete {
        let sizes = map
            .iter()
            .map(|(k, v)| (k.clone(), v.len() as u64))
            .collect();
        store.authority.check_write_quota(
            &grant.id,
            req.key,
            &sizes,
            req.body.len() as u64,
            &[],
        )?;
    }
    match req.kind {
        MutationKind::Delete => {
            map.remove(req.key);
        }
        _ => {
            map.insert(req.key.into(), req.body.to_vec());
        }
    }
    Ok(false)
}
