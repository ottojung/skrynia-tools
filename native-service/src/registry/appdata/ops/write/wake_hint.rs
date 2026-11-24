//! Durable wake slot before the source CAS; per-key hints only after commit.
use super::{Disk, push};

/// Advance the fixed wake slots under the mutation lock. O(1) storage with no
/// per-key hint quota, so a durable invalidation survives an offline sender.
/// Failure aborts the write: a committed mutation must never lack its wake.
pub fn precommit<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    wake_ns: &[u8],
) -> Result<Option<u64>, push::PushErr<D::Err, ()>> {
    // Every exact subscribed data key gets a durable fallback BEFORE commit.
    // Successful legacy hint enqueue clears it; a full/corrupt queue or
    // crash before enqueue leaves one discoverable coalesced invalidation.
    let slot = if key == push::WAKE_KEY {
        None
    } else {
        push::stage_data_slot(disk, base, ns, key)?
    };
    push::mark_wake_dirty(disk, base, ns, push::WAKE_KEY)?;
    if wake_ns != ns {
        push::mark_wake_dirty(disk, base, wake_ns, push::WAKE_KEY)?;
    }
    Ok(slot)
}

/// The original per-key hash hint is preferred when durably staged post-CAS.
/// If unavailable, keep the precommit dirty slot: a sender which already
/// read old hints may then ACK those without losing this newer change.
pub fn postcommit<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    hash: Option<[u8; 32]>,
    slot: Option<u64>,
) {
    if key == push::WAKE_KEY || slot.is_none() || !push::can_enqueue(disk, base, ns, key) {
        return;
    }
    if push::enqueue_hints(disk, base, ns, key, hash).is_ok()
        && let Some(g) = slot
    {
        let _ = push::wake_ack::<D, ()>(disk, base, ns, key, g);
    }
}
