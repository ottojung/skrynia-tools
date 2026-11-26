//! Length-prefixed i32-LE frame helpers shared by all descriptor handlers.
pub fn word(f: &[u8], at: &mut usize) -> Option<u32> {
    let raw: [u8; 4] = f.get(*at..*at + 4)?.try_into().ok()?;
    *at += 4;
    Some(u32::from_le_bytes(raw))
}

pub fn reply(code: i32, data: &[u8]) -> Vec<u8> {
    let mut out = code.to_le_bytes().to_vec();
    if code >= 0 {
        out.extend_from_slice(data)
    }
    out
}

/// A frame is valid only when every declared field was present and no byte is
/// left over. Handlers call this *before* any kernel effect.
pub fn complete(f: &[u8], at: usize) -> bool {
    at == f.len()
}
