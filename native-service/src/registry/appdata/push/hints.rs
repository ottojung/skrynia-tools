// External sender consumption: hints are durable files under
// pushsub/<target read cap>/<key hash>/hints/<seq>.bin. A narrowly scoped
// sender with read/write access to that subtree lists, reads, and deletes
// (acks) hint files; delivery uses independent VAPID/anti-SSRF outside P1.
use super::Disk;
use super::MAX_HINTS;
use super::err::PushErr;
use super::id;
use super::paths::{hint_path, hints_root, subs_dir};
use crate::hash_bytes;

pub fn enqueue_hints<D: Disk>(
    disk: &D,
    base: &str,
    read_cap: &[u8],
    key: &[u8],
    new_hash: Option<[u8; 32]>,
) -> Result<(), PushErr<D::Err, ()>> {
    let dir = subs_dir(base, read_cap, key);
    let entries = disk.list(&dir).map_err(PushErr::Disk)?;
    if entries.is_empty() {
        return Ok(());
    }
    let hdir = hints_root(base, read_cap, key);
    let existing = disk.list(&hdir).map_err(PushErr::Disk)?;
    if existing.len() >= MAX_HINTS {
        return Err(PushErr::Quota);
    }
    let mut max_seq = 0u64;
    for name in &existing {
        if let Some(seq) = parse_seq(name) {
            if seq > max_seq {
                max_seq = seq;
            }
        }
    }
    let seq = max_seq.saturating_add(1);
    let body = new_hash.unwrap_or([0u8; 32]).to_vec();
    let path = hint_path(base, read_cap, key, seq);
    disk.write_atomic(&path, &body).map_err(PushErr::Disk)?;
    let _ = hash_bytes(key);
    let _ = id::to_hex(read_cap);
    Ok(())
}

pub(crate) fn parse_seq(name: &str) -> Option<u64> {
    let stem = name.strip_suffix(".bin")?;
    stem.parse::<u64>().ok()
}
