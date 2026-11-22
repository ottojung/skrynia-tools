//! Opaque 32-byte hint read and idempotent ack for a scoped sender.
use super::{CAP_LEN, Disk, PushErr, hint_path, scope};

pub fn hint_read<D: Disk>(
    disk: &D,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<Vec<u8>, PushErr<D::Err, ()>> {
    scope(disk, base, sender_cap, ns, key)?;
    let bytes = disk
        .read_all(&hint_path(base, ns, key, seq))
        .map_err(PushErr::Disk)?
        .ok_or(PushErr::Missing)?;
    if bytes.len() != CAP_LEN {
        return Err(PushErr::BadKey);
    }
    Ok(bytes)
}

/// Ack is idempotent: a replayed seq that is already gone succeeds.
pub fn hint_ack<D: Disk>(
    disk: &D,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<(), PushErr<D::Err, ()>> {
    scope(disk, base, sender_cap, ns, key)?;
    let path = hint_path(base, ns, key, seq);
    if disk.read_all(&path).map_err(PushErr::Disk)?.is_none() {
        return Ok(());
    }
    disk.remove(&path).map_err(PushErr::Disk)
}
