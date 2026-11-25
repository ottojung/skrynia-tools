//! Single-slot write-ahead intent that bridges marker and index commit.
use super::{Disk, KEY_LEN, Scope, StoreErr, ensure_program, format};

pub fn write<D: Disk>(disk: &D, s: Scope, key: [u8; KEY_LEN]) -> Result<(), StoreErr<D::Err>> {
    let mut b = Vec::with_capacity(KEY_LEN + 1);
    b.extend_from_slice(&key);
    b.push(1);
    disk.write_atomic(&s.pending(), &format::seal(b))
        .map_err(StoreErr::Disk)
}

pub fn take<D: Disk>(disk: &D, s: Scope) -> Result<Option<[u8; KEY_LEN]>, StoreErr<D::Err>> {
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

pub fn clear<D: Disk>(disk: &D, s: Scope) -> Result<(), StoreErr<D::Err>> {
    disk.remove(&s.pending()).map_err(StoreErr::Disk)
}

pub fn scope(dir: &str) -> Scope<'_> {
    Scope { base: dir }
}

pub fn recover<D: Disk>(disk: &D, dir: &str) -> Result<(), StoreErr<D::Err>> {
    ensure_program(disk, dir)?;
    let s = scope(dir);
    if let Some(iid) = super::take_pending(disk, s)? {
        if super::read_active(disk, dir, &iid)?.is_some() {
            super::insert(disk, s, iid)?;
        } else {
            super::remove(disk, s, iid)?;
        }
        super::clear_pending(disk, s)?;
    }
    Ok(())
}
