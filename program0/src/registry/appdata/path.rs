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
    Some(format!("{dir}/{}.bin", id::to_hex(key)))
}
