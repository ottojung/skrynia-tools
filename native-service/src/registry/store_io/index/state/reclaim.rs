//! Bounded retired-node ledger; a crash leaves at most one transaction's nodes.
use super::{Disk, Scope, StoreErr, format, read};

fn path(s: Scope) -> String {
    format!("{}/index/retired", s.base.trim_end_matches('/'))
}

pub fn take<D: Disk>(disk: &D, s: Scope) -> Result<(), StoreErr<D::Err>> {
    let Some(raw) = read(disk, path(s))? else {
        return Ok(());
    };
    let Some(b) = format::open(&raw) else {
        return Err(StoreErr::BadRecord);
    };
    if b.len() != 12 {
        return Err(StoreErr::BadRecord);
    }
    for i in 0..3 {
        let id = u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
        if id != 0 {
            let p = match i {
                0 => s.root(id),
                1 => s.dir(id),
                _ => s.leaf(id),
            };
            disk.remove(&p).map_err(StoreErr::Disk)?;
        }
    }
    disk.remove(&path(s)).map_err(StoreErr::Disk)
}

pub fn save<D: Disk>(disk: &D, s: Scope, ids: [u32; 3]) -> Result<(), StoreErr<D::Err>> {
    let mut b = Vec::with_capacity(12);
    for id in ids {
        b.extend_from_slice(&id.to_le_bytes());
    }
    disk.write_atomic(&path(s), &format::seal(b))
        .map_err(StoreErr::Disk)
}
