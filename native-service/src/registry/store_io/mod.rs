mod active;
mod blob;
mod ready;
use super::cap::Disk;
use crate::id;

pub use active::{read_active, read_record, remove_active, write_active, write_record};
pub use blob::{ensure_blob, read_blob};
pub use ready::{is_ready, list as list_ready, seal as seal_ready};

#[derive(Debug, Clone)]
pub enum StoreErr<E> {
    Disk(E),
    BadId,
    Missing,
    BadRecord,
}

pub fn read_code<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
) -> Result<Vec<u8>, StoreErr<D::Err>> {
    let Some(h) = read_active(disk, dir, invoke_id)? else {
        return Err(StoreErr::Missing);
    };
    read_blob(disk, dir, &h)
}

pub fn del<D: Disk>(disk: &D, dir: &str, invoke_id: &[u8]) -> Result<(), StoreErr<D::Err>> {
    id::validate(invoke_id).map_err(|_| StoreErr::BadId)?;
    remove_active(disk, dir, invoke_id)
}

pub fn put_code<D: Disk>(
    disk: &D,
    dir: &str,
    invoke_id: &[u8],
    code: &[u8],
) -> Result<(), StoreErr<D::Err>> {
    let h = ensure_blob(disk, dir, code)?;
    write_active(disk, dir, invoke_id, &h)
}

pub fn get<D: Disk>(disk: &D, dir: &str, invoke_id: &[u8]) -> Result<Vec<u8>, StoreErr<D::Err>> {
    read_code(disk, dir, invoke_id)
}
