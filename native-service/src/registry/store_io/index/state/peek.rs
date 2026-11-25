//! Read-only pending intent accessor.
use super::{Disk, KEY_LEN, Scope, StoreErr, format};

pub fn peek<D: Disk>(disk: &D, s: Scope) -> Result<Option<[u8; KEY_LEN]>, StoreErr<D::Err>> {
    let Some(raw) = disk.read_all(&s.pending()).map_err(StoreErr::Disk)? else {
        return Ok(None);
    };
    let Some(b) = format::open(&raw) else {
        return Err(StoreErr::BadRecord);
    };
    if b.len() != KEY_LEN + 1 {
        return Err(StoreErr::BadRecord);
    }
    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&b[..KEY_LEN]);
    Ok(Some(key))
}
