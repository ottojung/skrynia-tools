//! Fixed-slot durable dirty generation for the reserved wake key.
//! One atomic state per exact (namespace, wake_key); no per-generation files,
//! no pruning, O(1) storage under an offline sender or a remove-denied Disk.
use super::{Disk, PushErr, wake_slot_path};

const MAGIC: &[u8; 4] = b"WSL1";
const LEN: usize = 16;

fn encode(generation: u64, dirty: bool) -> [u8; LEN] {
    let mut b = [0u8; LEN];
    b[..4].copy_from_slice(MAGIC);
    b[4..12].copy_from_slice(&generation.to_le_bytes());
    b[12] = u8::from(dirty);
    b
}

fn decode<D, L>(raw: Option<Vec<u8>>) -> Result<(u64, bool), PushErr<D, L>> {
    match raw {
        None => Ok((0, false)),
        Some(b)
            if b.len() == LEN
                && &b[..4] == MAGIC
                && b[12] <= 1
                && b[13..].iter().all(|x| *x == 0) =>
        {
            Ok((u64::from_le_bytes(b[4..12].try_into().unwrap()), b[12] == 1))
        }
        Some(_) => Err(PushErr::BadKey),
    }
}

/// Advance and mark dirty before the source commit; failure aborts the write.
pub fn mark_dirty<D: Disk>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<u64, PushErr<D::Err, ()>> {
    let path = wake_slot_path(base, ns, key);
    let (last, _) = decode(disk.read_all(&path).map_err(PushErr::Disk)?)?;
    let next = last.checked_add(1).ok_or(PushErr::Quota)?;
    disk.write_atomic(&path, &encode(next, true))
        .map_err(PushErr::Disk)?;
    Ok(next)
}

/// Current sender-visible generation and dirty flag, or an error if corrupt.
pub fn current<D: Disk, L>(
    disk: &D,
    base: &str,
    ns: &[u8],
    key: &[u8],
) -> Result<(u64, bool), PushErr<D::Err, L>> {
    decode(
        disk.read_all(&wake_slot_path(base, ns, key))
            .map_err(PushErr::Disk)?,
    )
}

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
