use super::data::State;
use super::{Authority, CompatError, CompatResult};

pub fn check_meta_add(authority: &Authority, acting: &str, state: &State) -> CompatResult<()> {
    check_delta(authority, acting, state, 2048)
}
pub fn check_user_add(
    authority: &Authority,
    grant_id: &str,
    users: usize,
    state: &State,
) -> CompatResult<()> {
    if users == 0 {
        return Ok(());
    }
    check_delta(authority, grant_id, state, 128)
}
fn check_delta(authority: &Authority, start: &str, state: &State, delta: u64) -> CompatResult<()> {
    let err = || CompatError::new(507, "grant_quota_exceeded");
    let sizes = state
        .objects
        .iter()
        .map(|(k, v)| (k.clone(), v.len() as u64))
        .collect();
    let mut cursor = Some(start);
    let mut steps = 0;
    while let Some(id) = cursor {
        let grant = authority.grant(id).ok_or_else(err)?;
        let usage = authority.usage(id, &sizes, &[]).ok_or_else(err)?;
        if grant
            .max_bytes
            .is_some_and(|m| usage.bytes.saturating_add(delta) > m)
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
