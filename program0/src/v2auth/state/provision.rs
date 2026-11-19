use super::cap;
use super::{Authority, Disk, StateErr, paths};
pub fn ensure_root<D: Disk, R: cap::Random<Err = D::Err>>(
    disk: &D,
    random: &R,
) -> Result<(), StateErr<D::Err>> {
    paths::ensure(disk).map_err(StateErr::Disk)?;
    let _guard = super::lock::acquire().map_err(|_| StateErr::Uncertain)?;
    if super::is_uncertain(disk).map_err(StateErr::Disk)? {
        return Err(StateErr::Uncertain);
    }
    let saved = disk.read_all(paths::AUTH).map_err(StateErr::Disk)?;
    let secret = super::secret::root(disk, random, saved.is_some())?;
    let hash = paths::hex32(&paths::sha256(&secret));
    if let Some(bytes) = saved {
        let auth = Authority::decode(&bytes).ok_or(StateErr::Decode)?;
        if !auth.users.iter().any(|u| u.hash == hash) {
            return Err(StateErr::Uncertain);
        }
        return Ok(());
    }
    disk.write_atomic(paths::AUTH, &Authority::root(hash).encode())
        .map_err(StateErr::Disk)
}
