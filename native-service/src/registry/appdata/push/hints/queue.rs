//! Legacy exact-key hash/delete hint queue, staged after committed source CAS.
//! The durable dirty-generation fallback is staged BEFORE commit so that
//! quota/corruption cannot abort a data write or lose a newer invalidation.
//! Senders MUST re-read authoritative state; hints are not event logs.
use super::{Disk, MAX_HINTS, PushErr, hint_path, hints_root, subs_dir};

pub fn enqueue_hints<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
    new_hash: Option<[u8; 32]>,
) -> Result<(), PushErr<D::Err, ()>> {
    let entries = disk
        .list(&subs_dir(base, read_cap, key))
        .map_err(PushErr::Disk)?;
    if entries.is_empty() {
        return Ok(());
    }
    let hdir = hints_root(base, read_cap, key);
    let existing = disk.list(&hdir).map_err(PushErr::Disk)?;
    if existing.len() >= MAX_HINTS {
        return Err(PushErr::Quota);
    }
    let seq = super::seq::reserve(disk, base, read_cap, key, &existing)?;
    let body = new_hash.unwrap_or([0u8; 32]);
    disk.write_atomic(&hint_path(base, read_cap, key, seq), &body)
        .map_err(PushErr::Disk)
}

/// Mark the reserved wake key dirty under the caller's mutation lock. The
/// fixed slot is the only durable wake state: generation advances before the
/// source commit, and no hint files are created or removed. See hints/wake.rs.
pub fn mark_wake_dirty<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
) -> Result<(), PushErr<D::Err, ()>> {
    let entries = disk
        .list(&subs_dir(base, read_cap, key))
        .map_err(PushErr::Disk)?;
    if entries.is_empty() {
        return Ok(());
    }
    super::wake::mark_dirty(disk, base, read_cap, key).map(|_| ())
}
