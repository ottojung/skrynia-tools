use super::util::{matches_precond, with_lock};
use super::{AppErr, CAP_LEN, Disk, MutLock, WriteAuth, io, valid_key};
use crate::wire::AppPrecond;

pub fn put<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    authz: WriteAuth<'_>,
    key: &[u8],
    expect: &AppPrecond,
    value: &[u8],
) -> Result<[u8; 32], AppErr<D::Err, L::Err>> {
    if !valid_key(key) {
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
        let hint = |e: super::push::PushErr<D::Err, ()>| match e {
            super::push::PushErr::Disk(d) => AppErr::Disk(d),
            super::push::PushErr::Busy | super::push::PushErr::Quota => AppErr::Busy,
            _ => AppErr::BadAuth,
        };
        if key == super::push::WAKE_KEY {
            // The reserved wake key uses the fixed dirty slot, even when a
            // client writes the key itself; legacy per-seq hint files would be
            // invisible to the WAKE_KEY sender special-case.
            super::push::mark_wake_dirty(disk, base, &ns, key).map_err(hint)?;
        } else {
            super::push::enqueue_hints(disk, base, &ns, key, Some(hash)).map_err(hint)?;
            super::push::mark_wake_dirty(disk, base, &ns, super::push::WAKE_KEY).map_err(hint)?;
        }
        io::write_value(disk, base, &ns, key, value).map_err(AppErr::from_store)
    })
}
