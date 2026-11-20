use super::{Disk, StoreErr, read_record};
use crate::store::{self, ActiveState};

pub fn write_record<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    next: &ActiveState,
) -> Result<(), StoreErr<D::Err>> {
    let path = store::active_path(dir, iid).ok_or(StoreErr::BadId)?;
    disk.write_atomic(&path, &store::encode_active(next))
        .map_err(StoreErr::Disk)
}
fn next_rev<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<u64, StoreErr<D::Err>> {
    read_record(disk, dir, iid)?
        .map(|s| s.revision.checked_add(1).ok_or(StoreErr::BadRecord))
        .unwrap_or(Ok(1))
}
pub fn write_active<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    hash: &[u8; 32],
) -> Result<(), StoreErr<D::Err>> {
    let revision = next_rev(disk, dir, iid)?;
    write_record(
        disk,
        dir,
        iid,
        &ActiveState {
            hash: *hash,
            revision,
            live: true,
        },
    )
}
pub fn remove_active<D: Disk>(disk: &D, dir: &str, iid: &[u8]) -> Result<(), StoreErr<D::Err>> {
    let revision = next_rev(disk, dir, iid)?;
    write_record(
        disk,
        dir,
        iid,
        &ActiveState {
            hash: [0; 32],
            revision,
            live: false,
        },
    )
}
