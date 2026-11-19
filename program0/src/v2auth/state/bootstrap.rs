use super::{Authority, Disk, StateErr, paths};
pub fn bootstrap_root<D: Disk>(disk: &D, token: &[u8; 32]) -> Result<(), StateErr<D::Err>> {
    paths::ensure(disk).map_err(StateErr::Disk)?;
    let _guard = super::lock::acquire().map_err(|_| StateErr::Uncertain)?;
    if super::is_uncertain(disk).map_err(StateErr::Disk)? {
        return Err(StateErr::Uncertain);
    }
    if disk
        .read_all(paths::AUTH)
        .map_err(StateErr::Disk)?
        .is_some()
    {
        return Err(StateErr::Exists);
    }
    let auth = Authority::root(paths::hex32(&paths::sha256(token)));
    disk.write_atomic(paths::AUTH, &auth.encode())
        .map_err(StateErr::Disk)
}
