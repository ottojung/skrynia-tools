//! Persistent sequence high-water survives ACK and crash/restart.
use super::{Disk, PushErr, hint_counter_path};

pub fn reserve<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
    existing: &[String],
) -> Result<u64, PushErr<D::Err, ()>> {
    let path = hint_counter_path(base, read_cap, key);
    let old = disk.read_all(&path).map_err(PushErr::Disk)?;
    let last = match old {
        Some(b) if b.len() == 8 => u64::from_le_bytes(b.try_into().unwrap()),
        Some(_) => return Err(PushErr::BadKey),
        None => existing
            .iter()
            .filter_map(|s| parse_seq(s))
            .max()
            .unwrap_or(0),
    };
    let next = last.checked_add(1).ok_or(PushErr::Quota)?;
    disk.write_atomic(&path, &next.to_le_bytes())
        .map_err(PushErr::Disk)?;
    Ok(next)
}

pub(crate) fn parse_seq(name: &str) -> Option<u64> {
    name.strip_suffix(".bin")?.parse::<u64>().ok()
}
