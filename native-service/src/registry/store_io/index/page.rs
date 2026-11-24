//! Keyset page entry point: load meta and root, then walk bounded nodes.
use super::{Disk, KEY_LEN, Scope, StoreErr, format, read};

pub fn page<D: Disk>(
    disk: &D,
    s: Scope,
    after: &Option<[u8; KEY_LEN]>,
    limit: u16,
) -> Result<Vec<[u8; KEY_LEN]>, StoreErr<D::Err>> {
    let Some(meta) = read(disk, s.meta())? else {
        return Ok(Vec::new());
    };
    let meta = format::open(&meta).ok_or(StoreErr::BadRecord)?;
    let (_, _, generation) = format::parse_meta(&meta).ok_or(StoreErr::BadRecord)?;
    let Some(raw) = read(disk, s.root(generation))? else {
        return Ok(Vec::new());
    };
    let root = format::open(&raw).ok_or(StoreErr::BadRecord)?;
    super::walk::walk(
        disk,
        s,
        &root,
        after.unwrap_or([0; KEY_LEN]),
        after.as_ref(),
        limit,
    )
}
