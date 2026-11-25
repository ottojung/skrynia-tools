//! Meta/node/slot helpers with fail-closed node validation.
use super::{Disk, KEY_LEN, Scope, StoreErr, find, format, read};

pub fn meta<D: Disk>(disk: &D, s: Scope) -> Result<(u32, u32, u32), StoreErr<D::Err>> {
    match read(disk, s.meta())? {
        Some(b) => format::open(&b)
            .and_then(|x| format::parse_meta(&x))
            .ok_or(StoreErr::BadRecord),
        None => Ok((0, 0, 0)),
    }
}

pub fn node<D: Disk>(
    disk: &D,
    p: String,
    cap: usize,
    step: usize,
) -> Result<Vec<u8>, StoreErr<D::Err>> {
    let b = read(disk, p)?.ok_or(StoreErr::Missing)?;
    let body = format::open(&b).ok_or(StoreErr::BadRecord)?;
    if body.len() != cap * step {
        return Err(StoreErr::BadRecord);
    }
    let mut last: Option<&[u8]> = None;
    for i in 0..cap {
        if step == KEY_LEN + 4 && find::dir_leaf(&body, i) == Some(0) {
            continue;
        }
        if step == KEY_LEN + 1 && find::leaf_flag(&body, i) != Some(true) {
            continue;
        }
        let k = if step == KEY_LEN + 4 {
            find::dir_key(&body, i)
        } else {
            find::leaf_key(&body, i)
        }
        .ok_or(StoreErr::BadRecord)?;
        if last.is_some_and(|prev| prev >= k) {
            return Err(StoreErr::BadRecord);
        }
        last = Some(k);
    }
    Ok(body)
}

pub fn slot(es: &[super::mutate::DirEntry], k: &[u8; KEY_LEN]) -> usize {
    match es.binary_search_by(|e| e.0.cmp(k)) {
        Ok(i) => i,
        Err(0) => 0,
        Err(i) => i - 1,
    }
}
