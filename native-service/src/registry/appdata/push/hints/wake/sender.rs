//! Scoped sender surface: generation-conditional read and stale-safe ACK.
use super::{Disk, PushErr, current, decode, encode, wake_slot_path};

/// Opaque 32-byte invalidation only for the exact current dirty generation.
pub fn read<D: Disk, L>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<Option<[u8; 32]>, PushErr<D::Err, L>> {
    let (generation, dirty) = current(disk, base, ns, key)?;
    Ok(if dirty && generation == seq {
        Some([0u8; 32])
    } else {
        None
    })
}

/// Clear dirty only for an exact current generation; a stale ACK is a no-op.
pub fn ack<D: Disk, L>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
    seq: u64,
) -> Result<(), PushErr<D::Err, L>> {
    let path = wake_slot_path(base, ns, key);
    let (generation, dirty) = decode(disk.read_all(&path).map_err(PushErr::Disk)?)?;
    if !dirty || generation != seq {
        return Ok(());
    }
    disk.write_atomic(&path, &encode(generation, false))
        .map_err(PushErr::Disk)
}
