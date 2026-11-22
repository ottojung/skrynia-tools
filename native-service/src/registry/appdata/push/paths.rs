use super::id;
use crate::hash_bytes;

pub const SUBS_DIR: &str = "subs";
pub const HINTS_DIR: &str = "hints";

pub fn target_root(base: &str, read_cap: &[u8]) -> String {
    format!(
        "{}/pushsub/{}",
        base.trim_end_matches('/'),
        id::to_hex(read_cap)
    )
}

pub fn object_dir(base: &str, read_cap: &[u8], key: &[u8]) -> String {
    format!(
        "{}/{}",
        target_root(base, read_cap),
        id::to_hex(&hash_bytes(key))
    )
}

pub fn subs_dir(base: &str, read_cap: &[u8], key: &[u8]) -> String {
    format!("{}/{}", object_dir(base, read_cap, key), SUBS_DIR)
}

pub fn sub_path(base: &str, read_cap: &[u8], key: &[u8], subscriber: &[u8]) -> String {
    format!(
        "{}/{}.bin",
        subs_dir(base, read_cap, key),
        id::to_hex(&hash_bytes(subscriber))
    )
}

pub fn hints_root(base: &str, read_cap: &[u8], key: &[u8]) -> String {
    format!("{}/{}", object_dir(base, read_cap, key), HINTS_DIR)
}

pub fn hint_path(base: &str, read_cap: &[u8], key: &[u8], seq: u64) -> String {
    format!("{}/{:020}.bin", hints_root(base, read_cap, key), seq)
}

/// Monotone high-water mark is distinct from deletable pending hints.
pub fn hint_counter_path(base: &str, read_cap: &[u8], key: &[u8]) -> String {
    format!("{}/hint-sequence.bin", object_dir(base, read_cap, key))
}
