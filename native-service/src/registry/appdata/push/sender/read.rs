//! Opaque 32-byte hint read and idempotent ack for a scoped sender. Wake-key
//! reads and ACKs address the fixed dirty slot; data keys keep hint files.
use super::{
    CAP_LEN, Disk, MutLock, PushErr, WAKE_KEY, hint_path, scope, wake_ack, wake_read, with_lock,
};

pub fn hint_read<D: Disk, L: MutLock>(
    disk: &D,
    lock: &L,
    base: &str,
    sender_cap: &[u8],
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<Vec<u8>, PushErr<D::Err, L::Err>> {
    with_lock(lock, || {
        scope(disk, base, sender_cap, ns, key)?;
        if key == WAKE_KEY {
            return wake_read(disk, base, ns, key, seq)?
                .ok_or(PushErr::Missing)
                .map(|bytes| bytes.to_vec());
        }
        let bytes = disk
            .read_all(&hint_path(base, ns, key, seq))
            .map_err(PushErr::Disk)?
            .ok_or(PushErr::Missing)?;
        if bytes.len() != CAP_LEN {
            return Err(PushErr::BadKey);
        }
        Ok(bytes)
    })
}

/// Ack is idempotent: a replayed or superseded seq succeeds without clearing
/// a newer generation.
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
        let path = hint_path(base, ns, key, seq);
        if disk.read_all(&path).map_err(PushErr::Disk)?.is_none() {
            return Ok(());
        }
        disk.remove(&path).map_err(PushErr::Disk)
    })
}
