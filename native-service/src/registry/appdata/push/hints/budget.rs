//! Avoid scanning huge legacy hint directories inside fuel-limited WASM.
use super::{Disk, hint_counter_path};

/// Historical exact-key hash hints remain available for first 256 writes.
/// Later changes use the durable coalesced fallback exclusively, independent
/// of sender timing, disk directory size, or a preexisting saturated queue.
pub const MAX_LEGACY_HINTS_PER_KEY: u64 = 256;

pub fn can_enqueue<D: Disk>(disk: &D, base: &str, ns: &[u8], key: &[u8]) -> bool {
    let path = hint_counter_path(base, ns, key);
    match disk.read_all(&path) {
        Ok(Some(bytes)) if bytes.len() == 8 => {
            u64::from_le_bytes(bytes.try_into().unwrap()) < MAX_LEGACY_HINTS_PER_KEY
        }
        // Original per-key format initializes its counter on first write.
        // Existing legacy entries without a counter are handled by the
        // durable fallback after one attempted enqueue; never guess a seq.
        Ok(None) => true,
        _ => false,
    }
}
