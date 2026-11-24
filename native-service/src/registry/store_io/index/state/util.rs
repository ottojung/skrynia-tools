//! Meta/node/slot helpers for index path loading.
use super::{Disk, KEY_LEN, Scope, StoreErr, find, format, mutate, read};

pub fn meta<D: Disk>(disk: &D, s: Scope) -> Result<(u32, u32, u32), StoreErr<D::Err>> {
    match read(disk, s.meta())? {
        Some(b) => format::open(&b)
            .and_then(|x| format::parse_meta(&x))
            .ok_or(StoreErr::BadRecord),
        None => Ok((0, 0, 0)),
    }
}

pub fn node<D: Disk>(disk: &D, p: String) -> Result<Vec<u8>, StoreErr<D::Err>> {
    match read(disk, p)? {
        Some(b) => format::open(&b).ok_or(StoreErr::BadRecord),
        None => Ok(find::empty_dir()),
    }
}

pub fn slot(es: &[mutate::DirEntry], k: &[u8; KEY_LEN]) -> usize {
    match es.binary_search_by(|e| e.0.cmp(k)) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) => i - 1,
    }
}
