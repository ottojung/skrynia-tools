use sha2::{Digest, Sha256};
use std::path::Path;

/// Unambiguously encode kind, opaque relative path and exact content size.
pub fn add(hash: &mut Sha256, kind: u8, relative: &Path, size: u64) {
    let name = relative.as_os_str().as_encoded_bytes();
    hash.update([kind]);
    hash.update((name.len() as u64).to_be_bytes());
    hash.update(name);
    hash.update(size.to_be_bytes());
}
