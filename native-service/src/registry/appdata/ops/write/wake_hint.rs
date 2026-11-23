//! Durable wake slot before the source CAS; per-key hints only after commit.
use super::{Disk, push};

/// Advance the fixed wake slots under the mutation lock. O(1) storage with no
/// per-key hint quota, so a durable invalidation survives an offline sender.
/// Failure aborts the write: a committed mutation must never lack its wake.
pub fn precommit<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    wake_ns: &[u8],
) -> Result<(), push::PushErr<D::Err, ()>> {
    push::mark_wake_dirty(disk, base, ns, push::WAKE_KEY)?;
    if wake_ns != ns {
        push::mark_wake_dirty(disk, base, wake_ns, push::WAKE_KEY)?;
    }
    Ok(())
}

/// Best-effort per-key hint after the source CAS committed. A full queue is
/// degraded notification, never a data error: the precommit wake slot is
/// already durable, so no wake is lost and no hint names uncommitted data.
pub fn postcommit<D: Disk>(disk: &D, base: &str, ns: &[u8], key: &[u8], hash: Option<[u8; 32]>) {
    if key != push::WAKE_KEY {
        let _ = push::enqueue_hints(disk, base, ns, key, hash);
    }
}
