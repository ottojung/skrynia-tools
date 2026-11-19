use super::id;

pub fn ns_dir(base: &str, read_cap: &[u8]) -> Option<String> {
    id::validate(read_cap).ok()?;
    Some(format!(
        "{}/appdata/{}",
        base.trim_end_matches('/'),
        id::to_hex(read_cap)
    ))
}

pub fn valid_key(key: &[u8]) -> bool {
    !key.is_empty() && key.len() <= 256
}

pub fn data_path(base: &str, read_cap: &[u8], key: &[u8]) -> Option<String> {
    if !valid_key(key) {
        return None;
    }
    let dir = ns_dir(base, read_cap)?;
    // Hash opaque object-key bytes before pathname encoding: raw hex could
    // exceed a filesystem's 255-byte component limit even for valid keys.
    Some(format!("{dir}/{}.bin", id::to_hex(&crate::hash_bytes(key))))
}
