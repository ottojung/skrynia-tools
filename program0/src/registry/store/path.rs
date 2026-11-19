use crate::id;

pub fn active_path(dir: &str, invoke_id: &[u8]) -> Option<String> {
    id::validate(invoke_id).ok()?;
    Some(format!(
        "{}/active/{}.bin",
        dir.trim_end_matches('/'),
        id::to_hex(invoke_id)
    ))
}

pub fn blob_path(dir: &str, code_hash: &[u8]) -> Option<String> {
    if code_hash.len() != 32 {
        return None;
    }
    Some(format!(
        "{}/blob/{}.bin",
        dir.trim_end_matches('/'),
        id::to_hex(code_hash)
    ))
}
