use super::check::marker;
use super::*;
pub fn seal<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<(), StoreErr<D::Err>> {
    read_blob(disk, dir, hash).map(|_| ())?;
    if is_ready(disk, dir, iid, hash)? {
        return Ok(());
    }
    let p = store::ready_path(dir, iid, hash).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&p, &marker(hash)).map_err(StoreErr::Disk)
}
