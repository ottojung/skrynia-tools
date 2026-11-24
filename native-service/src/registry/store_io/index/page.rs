//! Keyset page entry point: load meta and root, then walk bounded nodes.
use super::{Disk, KEY_LEN, Scope, StoreErr, format, read};

pub fn page<D: Disk>(
    disk: &D,
    s: Scope,
    after: &Option<[u8; KEY_LEN]>,
    limit: u16,
) -> Result<Vec<[u8; KEY_LEN]>, StoreErr<D::Err>> {
    let meta = read(disk, s.meta())?.ok_or(StoreErr::Missing)?;
    let meta = format::open(&meta).ok_or(StoreErr::BadRecord)?;
    let (_, _, generation) = format::parse_meta(&meta).ok_or(StoreErr::BadRecord)?;
    let raw = read(disk, s.root(generation))?.ok_or(StoreErr::Missing)?;
    let root = format::open(&raw).ok_or(StoreErr::BadRecord)?;
    if root.len() != super::ROOT_CAP * (KEY_LEN + 4) {
        return Err(StoreErr::BadRecord);
    }
    super::walk::walk(
        disk,
        s,
        &root,
        after.unwrap_or([0; KEY_LEN]),
        after.as_ref(),
        limit,
    )
}
