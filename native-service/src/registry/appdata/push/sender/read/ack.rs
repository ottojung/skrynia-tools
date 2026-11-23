//! ACK an old seq without clearing a newer slot generation.
use super::*;
/// Idempotent ACK: cannot clear a newer fallback slot generation.
pub fn hint_ack<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<(), PushErr<D::Err, L::Err>> {
    with_lock(lock, || {
        scope(disk, base, sender_cap, ns, key)?;
        if key == WAKE_KEY {
            return wake_ack(disk, base, ns, key, seq);
        }
        if seq & DATA_SLOT_FLAG != 0 {
            return wake_ack(disk, base, ns, key, seq & !DATA_SLOT_FLAG);
        }
        let path = hint_path(base, ns, key, seq);
        if disk.read_all(&path).map_err(PushErr::Disk)?.is_none() {
            return Ok(());
        }
        disk.remove(&path).map_err(PushErr::Disk)
    })
}
