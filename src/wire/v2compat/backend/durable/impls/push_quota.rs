use super::{Authority, CompatError, CompatResult, State, sub_counts};

pub fn check(state: &State, grant_id: &str) -> CompatResult<()> {
    let err = || CompatError::new(507, "grant_quota_exceeded");
    let authority = Authority::from_grants(state.grants.clone())
        .map_err(|e| CompatError::new(500, format!("grant_graph:{e}")))?;
    let sizes = state
        .objects
        .iter()
        .map(|(k, v)| (k.clone(), v.len() as u64))
        .collect();
    let subs = sub_counts(state);
    let mut cursor = Some(grant_id);
    let mut steps = 0;
    while let Some(id) = cursor {
        let grant = authority.grant(id).ok_or_else(err)?;
        let usage = authority.usage(id, &sizes, &subs).ok_or_else(err)?;
        if grant
            .max_bytes
            .is_some_and(|m| usage.bytes.saturating_add(4096) > m)
            || grant
                .max_objects
                .is_some_and(|m| usage.objects.saturating_add(1) > m)
        {
            return Err(err());
        }
        cursor = grant.parent.as_deref();
        steps += 1;
        if steps > state.grants.len() {
            return Err(err());
        }
    }
    Ok(())
}
