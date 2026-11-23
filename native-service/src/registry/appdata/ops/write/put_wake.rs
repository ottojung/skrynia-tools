use super::util::{hint_err, matches_precond, with_lock};
use super::{AppErr, CAP_LEN, Disk, MutLock, WriteAuth, io, valid_key};
use crate::wire::AppPrecond;
#[allow(clippy::too_many_arguments)]
pub fn put_wake<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    authz: WriteAuth<'_>,
    key: &[u8],
    expect: &AppPrecond,
    value: &[u8],
    wake_grant: &[u8],
    wake_key: &[u8],
) -> Result<[u8; 32], AppErr<D::Err, L::Err>> {
    if !valid_key(key) || wake_key != super::push::WAKE_KEY || wake_grant.len() != CAP_LEN {
        return Err(AppErr::BadKey);
    }
    if let AppPrecond::Present(h) = expect
        && h.len() != CAP_LEN
    {
        return Err(AppErr::BadKey);
    }
    let ns = authz.namespace().map_err(|_| AppErr::BadAuth)?;
    with_lock(lock, || {
        let state = io::read_state(disk, base, &ns, key).map_err(AppErr::from_store)?;
        if !matches_precond(&state, expect) {
            return Err(AppErr::Conflict(state.hash()));
        }
        let hash = crate::hash_bytes(value);
        let wake_ns = super::push::verify_grant(
            disk,
            base,
            &super::push::Kind::Wake,
            wake_grant,
            wake_key,
            &ns,
        )
        .map_err(hint_err)?;
        super::wake_hint::precommit(disk, base, &ns, &wake_ns).map_err(hint_err)?;
        io::write_value(disk, base, &ns, key, value).map_err(AppErr::from_store)?;
        super::wake_hint::postcommit(disk, base, &ns, key, Some(hash));
        Ok(hash)
    })
}
