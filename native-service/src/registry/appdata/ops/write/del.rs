use super::util::{hint_err, with_lock};
use super::{AppErr, CAP_LEN, Disk, MutLock, State, WriteAuth, io, valid_key};
use crate::hash_bytes;

pub fn delete<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    authz: WriteAuth<'_>,
    key: &[u8],
    expect: &[u8],
) -> Result<(), AppErr<D::Err, L::Err>> {
    if !valid_key(key) || expect.len() != CAP_LEN {
        return Err(AppErr::BadKey);
    }
    let ns = authz.namespace().map_err(|_| AppErr::BadAuth)?;
    with_lock(lock, || {
        let state = io::read_state(disk, base, &ns, key).map_err(AppErr::from_store)?;
        match &state {
            State::Present(v) if hash_bytes(v).as_slice() == expect => {}
            _ => return Err(AppErr::Conflict(state.hash())),
        }
        let slot = super::wake_hint::precommit(disk, base, &ns, key, &ns).map_err(hint_err)?;
        io::delete_value(disk, base, &ns, key).map_err(AppErr::from_store)?;
        super::wake_hint::postcommit(disk, base, &ns, key, None, slot);
        Ok(())
    })
}
