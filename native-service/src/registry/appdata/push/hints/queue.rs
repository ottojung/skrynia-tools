//! Stage a durable change hint before the related object CAS commit.
//! Readers only inspect hints under the same mutation lock.
//! A hint is an invalidation, not proof of a committed value: if the following
//! object write fails, a spurious hint may remain. Senders MUST re-read the
//! authoritative object state after receiving a hint.
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

/// Coalesced durable dirty generation for a wake key, called under the same
/// mutation lock as the object write. The generation IS the hint seq: it is
/// monotone, so an ACK(g) can only remove the hint at g and can never clear a
/// newer generation at g' > g. Exactly one hint is pending per wake key, so
/// storage is O(1) and MAX_HINTS can never block an ordinary data-plane write.
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
    let hdir = hints_root(base, read_cap, key);
    let existing = disk.list(&hdir).map_err(PushErr::Disk)?;
    let generation = super::seq::reserve(disk, base, read_cap, key, &existing)?;
    // New generation is durable before any older hint is dropped. A crash in
    // between leaves only extra spurious hints, never a lost wake.
    disk.write_atomic(&hint_path(base, read_cap, key, generation), &[0u8; 32])
        .map_err(PushErr::Disk)?;
    for name in existing.iter().filter_map(|n| super::seq::parse_seq(n)) {
        if name != generation {
            disk.remove(&hint_path(base, read_cap, key, name))
                .map_err(PushErr::Disk)?;
        }
    }
    Ok(())
}
