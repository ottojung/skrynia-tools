use super::id;

pub fn app_dir(base: &str, app: &[u8]) -> Option<String> {
    id::validate(app).ok()?;
    Some(format!(
        "{}/appdata/{}",
        base.trim_end_matches('/'),
        id::to_hex(app)
    ))
}

pub fn key_hex(key: &[u8]) -> Option<String> {
    if key.is_empty() || key.len() > 256 {
        return None;
    }
    Some(id::to_hex(key))
}

pub fn data_path(base: &str, app: &[u8], key: &[u8]) -> Option<String> {
    let dir = app_dir(base, app)?;
    Some(format!("{dir}/{}.bin", key_hex(key)?))
}
