//! Reclaim fresh or retired nodes according to the committed generation.
use super::drop::drop_node;
use super::write_ops::{MAGIC, path};

pub fn recover<D: super::super::Disk>(
    disk: &D,
    s: super::super::Scope,
) -> Result<(), super::super::StoreErr<D::Err>> {
    let Some(raw) = super::super::read(disk, path(s))? else {
        return Ok(());
    };
    let Some(b) = super::super::format::open(&raw) else {
        return Err(super::super::StoreErr::BadRecord);
    };
    if b.len() != 40 || &b[..4] != MAGIC {
        return Err(super::super::StoreErr::BadRecord);
    }
    let (words, _) = b[4..].as_chunks::<4>();
    let n: Vec<u32> = words.iter().map(|c| u32::from_le_bytes(*c)).collect();
    let actual = match super::super::read(disk, s.meta())? {
        None => 0,
        Some(m) => {
            let body = super::super::format::open(&m).ok_or(super::super::StoreErr::BadRecord)?;
            super::super::format::parse_meta(&body)
                .ok_or(super::super::StoreErr::BadRecord)?
                .2
        }
    };
    let valid = if n[0] == 1 {
        actual <= 1
    } else {
        actual == n[0] || actual.checked_add(1) == Some(n[0])
    };
    if !valid {
        return Err(super::super::StoreErr::BadRecord);
    }
    if actual == n[0] {
        drop_node(disk, s, b'l', n[7])?;
        drop_node(disk, s, b'd', n[6])?;
        drop_node(disk, s, b'r', n[5])?;
    } else {
        drop_node(disk, s, b'l', n[1])?;
        drop_node(disk, s, b'l', n[2])?;
        drop_node(disk, s, b'd', n[3])?;
        drop_node(disk, s, b'd', n[4])?;
        drop_node(disk, s, b'r', n[0])?;
    }
    disk.remove(&path(s)).map_err(super::super::StoreErr::Disk)
}
