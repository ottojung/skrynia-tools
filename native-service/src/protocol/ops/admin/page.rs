use super::res;

pub fn encode(selected: &[[u8; 32]]) -> Vec<u8> {
    let mut body = Vec::with_capacity(2 + selected.len() * 32);
    body.extend_from_slice(&(selected.len() as u16).to_le_bytes());
    for hash in selected {
        body.extend_from_slice(hash);
    }
    res::ok(body)
}

pub type Cursor = Option<[u8; 32]>;

pub fn after(after: &Option<Vec<u8>>, limit: u16) -> Option<Cursor> {
    if limit == 0 || limit > 64 || after.as_ref().is_some_and(|v| v.len() != 32) {
        return None;
    }
    Some(match after {
        Some(v) => Some(v.as_slice().try_into().ok()?),
        None => None,
    })
}
