//! Bounded keyset listing over the immutable ready-version index.
use super::*;
use crate::index::{self, Scope};
use crate::store_io::StoreErr;

pub fn list<D: Disk>(
    disk: &D,
    dir: &str,
    iid: &[u8],
    after: Option<[u8; 32]>,
    limit: u16,
) -> Result<Vec<[u8; 32]>, StoreErr<D::Err>> {
    recover::recover(disk, dir, iid)?;
    let base = store::ready_dir(dir, iid).ok_or(StoreErr::BadId)?;
    if disk
        .read_all(&Scope { base: &base }.meta())
        .map_err(StoreErr::Disk)?
        .is_none()
    {
        return Ok(Vec::new());
    }
    index::page(disk, Scope { base: &base }, &after, limit)
}
