use super::{Disk, StoreErr};
use crate::store;

pub fn read_active<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
) -> Result<Option<[u8; 32]>, StoreErr<D::Err>> {
    let path = store::active_path(dir, invoke_id).ok_or(StoreErr::BadId)?;
    let Some(bytes) = disk.read_all(&path).map_err(StoreErr::Disk)? else {
        return Ok(None);
    };
    store::decode_active(&bytes)
        .map(Some)
        .ok_or(StoreErr::BadRecord)
}

pub fn write_active<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
    code_hash: &[u8; 32],
) -> Result<(), StoreErr<D::Err>> {
    let path = store::active_path(dir, invoke_id).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&path, &store::encode_active(code_hash))
        .map_err(StoreErr::Disk)
}

pub fn remove_active<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
) -> Result<(), StoreErr<D::Err>> {
    let path = store::active_path(dir, invoke_id).ok_or(StoreErr::BadId)?;
    disk.remove(&path).map_err(StoreErr::Disk)
}
