//! Scoped single-hint read, including a coalesced overflow generation.
use super::*;
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
        if seq & DATA_SLOT_FLAG != 0 {
            return wake_read(disk, base, ns, key, seq & !DATA_SLOT_FLAG)?
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
