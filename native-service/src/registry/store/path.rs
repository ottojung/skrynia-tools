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

pub fn ready_path(dir: &str, iid: &[u8], hash: &[u8]) -> Option<String> {
    if hash.len() != 32 {
        return None;
    }
    let base = ready_dir(dir, iid)?;
    Some(format!("{base}/{}.ready", id::to_hex(hash)))
}
pub fn ready_dir(dir: &str, iid: &[u8]) -> Option<String> {
    id::validate(iid).ok()?;
    Some(format!(
        "{}/ready/{}",
        dir.trim_end_matches('/'),
        id::to_hex(iid)
    ))
}
