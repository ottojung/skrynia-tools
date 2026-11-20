pub const ACTIVE_VER: u8 = 3;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveState {
    pub hash: [u8; 32],
    pub revision: u64,
    pub live: bool,
}
pub fn encode_active(s: &ActiveState) -> Vec<u8> {
    let mut out = Vec::with_capacity(42);
    out.push(ACTIVE_VER);
    out.push(u8::from(s.live));
    out.extend_from_slice(&s.hash);
    out.extend_from_slice(&s.revision.to_le_bytes());
    out
}
pub fn decode_active(bytes: &[u8]) -> Option<ActiveState> {
    if bytes.len() != 42 || bytes[0] != ACTIVE_VER || bytes[1] > 1 {
        return None;
    }
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&bytes[2..34]);
    let revision = u64::from_le_bytes(bytes[34..42].try_into().ok()?);
    if revision == 0 {
        return None;
    }
    Some(ActiveState {
        hash,
        revision,
        live: bytes[1] == 1,
    })
}
