pub const VERSION: u8 = 1;
pub const OP_PUBLISH: u8 = 1;
pub const OP_UNPUBLISH: u8 = 2;

pub struct Request {
    pub publish: bool,
    pub path: Vec<u8>,
}

/// `[version][32-byte token][op][u32 LE path length][exact path bytes]`
pub fn decode(raw: &[u8]) -> Option<Request> {
    if raw.len() < 38 || raw[0] != VERSION {
        return None;
    }
    let publish = match raw[33] {
        OP_PUBLISH => true,
        OP_UNPUBLISH => false,
        _ => return None,
    };
    let len = u32::from_le_bytes(raw[34..38].try_into().ok()?) as usize;
    if len == 0 || len > 4096 || raw.len() != 38 + len {
        return None;
    }
    Some(Request {
        publish,
        path: raw[38..].to_vec(),
    })
}

#[cfg(target_arch = "wasm32")]
pub fn reply(code: i32) -> [u8; 4] {
    code.to_le_bytes()
}
