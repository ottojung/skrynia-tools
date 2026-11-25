use super::check::marker;
use super::*;
use crate::index::{self, Scope};

pub fn seal<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<(), StoreErr<D::Err>> {
    read_blob(disk, dir, hash).map(|_| ())?;
    recover::recover(disk, dir, iid)?;
    let base = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let scope = Scope { base: &base };
    index::dry_run(disk, scope, *hash, true)?;
    if is_ready(disk, dir, iid, hash)? {
        index::insert(disk, scope, *hash)?;
        return Ok(());
    }
    index::preflight(disk, scope)?;
    index::write_pending(disk, scope, *hash)?;
    let p = store::ready_path(dir, iid, hash).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&p, &marker(hash))
        .map_err(StoreErr::Disk)?;
    index::insert(disk, scope, *hash)?;
    index::clear_pending(disk, scope)?;
    Ok(())
}
