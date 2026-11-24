//! Exact data-key fallback slot: durable invalidation BEFORE source commit.
use super::{Disk, PushErr, subs_dir};

pub fn stage_data_slot<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<Option<u64>, PushErr<D::Err, ()>> {
    let subs = disk.list(&subs_dir(base, ns, key)).map_err(PushErr::Disk)?;
    if subs.is_empty() {
        return Ok(None);
    }
    let (old, _) = super::wake::current::<D, ()>(disk, base, ns, key)?;
    if old >= (1u64 << 63) - 1 {
        return Err(PushErr::Quota);
    }
    super::wake::mark_dirty(disk, base, ns, key).map(Some)
}
