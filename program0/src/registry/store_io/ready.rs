use super::{Disk, StoreErr};
use crate::{store, store_ls};

fn marker(hash: &[u8; 32]) -> Vec<u8> {
    let mut out = vec![1];
    out.extend_from_slice(hash);
    out
}
pub fn is_ready<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<bool, StoreErr<D::Err>> {
    let p = store::ready_path(dir, iid, hash).ok_or(StoreErr::BadId)?;
    match disk.read_all(&p).map_err(StoreErr::Disk)? {
        None => Ok(false),
        Some(b) if b == marker(hash) => Ok(true),
        Some(_) => Err(StoreErr::BadRecord),
    }
}
pub fn seal<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<(), StoreErr<D::Err>> {
    store_io_check(disk, dir, hash)?;
    if is_ready(disk, dir, iid, hash)? {
        return Ok(());
    }
    let p = store::ready_path(dir, iid, hash).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&p, &marker(hash)).map_err(StoreErr::Disk)
}
fn store_io_check<D: Disk>(disk: &D, dir: &str, hash: &[u8; 32]) -> Result<(), StoreErr<D::Err>> {
    super::blob::read_blob(disk, dir, hash).map(|_| ())
}
pub fn list<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<Vec<[u8; 32]>, StoreErr<D::Err>> {
    let path = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    let names = disk.list(&path).map_err(StoreErr::Disk)?;
    let mut hashes = Vec::new();
    for name in names {
        let Some(hex) = name.strip_suffix(".ready") else {
            continue;
        };
        let Some(hash) = store_ls::parse_id_name(&format!("{hex}.bin")) else {
            continue;
        };
        let hash: [u8; 32] = hash.try_into().map_err(|_| StoreErr::BadId)?;
        if is_ready(disk, dir, iid, &hash)? {
            hashes.push(hash);
        }
    }
    hashes.sort();
    Ok(hashes)
}
