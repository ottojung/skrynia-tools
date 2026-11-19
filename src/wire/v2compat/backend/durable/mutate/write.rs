use super::{Authority, CompatResult, Mutation, MutationKind, State, data};

pub fn apply(
    state: &mut State,
    authority: &Authority,
    grant_id: &str,
    req: &Mutation<'_>,
) -> CompatResult<()> {
    if req.kind != MutationKind::Delete {
        let sizes = state
            .objects
            .iter()
            .map(|(k, v)| (k.clone(), v.len() as u64))
            .collect();
        let subs = data::sub_counts(state);
        authority.check_write_quota(grant_id, req.key, &sizes, req.body.len() as u64, &subs)?;
        state.objects.insert(req.key.into(), req.body.to_vec());
    } else {
        state.objects.remove(req.key);
    }
    Ok(())
}
