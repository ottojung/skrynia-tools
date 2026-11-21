use super::{Disk, StoreErr};
use crate::store::{self, ActiveState};

pub fn read_record<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
) -> Result<Option<ActiveState>, StoreErr<D::Err>> {
    let path = store::active_path(dir, invoke_id).ok_or(StoreErr::BadId)?;
    let Some(bytes) = disk.read_all(&path).map_err(StoreErr::Disk)? else {
        return Ok(None);
    };
    store::decode_active(&bytes)
        .ok_or(StoreErr::BadRecord)
        .map(Some)
}

pub fn read_active<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
) -> Result<Option<[u8; 32]>, StoreErr<D::Err>> {
    Ok(read_record(disk, dir, invoke_id)?
        .filter(|r| r.live)
        .map(|r| r.hash))
}
