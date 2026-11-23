use super::util::{hint_err, with_lock};
use super::{AppErr, CAP_LEN, Disk, MutLock, State, WriteAuth, io, valid_key};
use crate::hash_bytes;
#[allow(clippy::too_many_arguments)]
pub fn delete_wake<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    authz: WriteAuth<'_>,
    key: &[u8],
    expect: &[u8],
    wake_grant: &[u8],
    wake_key: &[u8],
) -> Result<(), AppErr<D::Err, L::Err>> {
    if !valid_key(key)
        || expect.len() != CAP_LEN
        || wake_key != super::push::WAKE_KEY
        || wake_grant.len() != CAP_LEN
    {
        return Err(AppErr::BadKey);
    }
    let ns = authz.namespace().map_err(|_| AppErr::BadAuth)?;
    with_lock(lock, || {
        let state = io::read_state(disk, base, &ns, key).map_err(AppErr::from_store)?;
        match &state {
            State::Present(v) if hash_bytes(v).as_slice() == expect => {}
            _ => return Err(AppErr::Conflict(state.hash())),
        }
        let wake_ns = super::push::verify_grant(
            disk,
            base,
            &super::push::Kind::Wake,
            wake_grant,
            wake_key,
            &ns,
        )
        .map_err(hint_err)?;
        super::wake_hint::stage(disk, base, &ns, key, None, &wake_ns).map_err(hint_err)?;
        io::delete_value(disk, base, &ns, key).map_err(AppErr::from_store)
    })
}
