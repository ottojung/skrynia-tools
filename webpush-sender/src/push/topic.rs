//! Deterministic per-hint push Topic: dedupe without cross-hint collapse.
use sha2::{Digest, Sha256};

/// Replay of the same seq+hint keeps one topic (push services dedupe it);
/// distinct hints never collapse onto the same push-service slot.
pub fn topic_for(seed: &[u8; 16], seq: u64, hint: &[u8]) -> [u8; 16] {
    let digest = Sha256::new()
        .chain_update(seed)
        .chain_update(seq.to_le_bytes())
        .chain_update(hint)
        .finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&digest[..16]);
    out
}
