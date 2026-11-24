//! Mutation-lock slot lifecycle: mark dirty before source commit.
use super::{Disk, PushErr, decode, encode, wake_slot_path};

/// Advance and mark dirty before the source commit; failure aborts the write.
pub fn mark_dirty<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<u64, PushErr<D::Err, ()>> {
    let path = wake_slot_path(base, ns, key);
    let (last, _) = decode(disk.read_all(&path).map_err(PushErr::Disk)?)?;
    let next = last.checked_add(1).ok_or(PushErr::Quota)?;
    disk.write_atomic(&path, &encode(next, true))
        .map_err(PushErr::Disk)?;
    Ok(next)
}

/// Current sender-visible generation and dirty flag, or an error if corrupt.
pub fn current<D: Disk, L>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<(u64, bool), PushErr<D::Err, L>> {
    decode(
        disk.read_all(&wake_slot_path(base, ns, key))
            .map_err(PushErr::Disk)?,
    )
}
