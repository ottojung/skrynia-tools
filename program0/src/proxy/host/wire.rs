pub const OPEN_DISK: u8 = 1;
pub const READ: u8 = 2;
pub const WRITE: u8 = 3;
pub const CLOSE: u8 = 4;
pub const LIMIT: usize = 64 * 1024;
pub const INVALID: i32 = -1;
pub const OVERFLOW: i32 = -11;

pub fn u32_at(req: &[u8], at: &mut usize) -> Option<u32> {
    let bytes: [u8; 4] = req.get(*at..(*at).checked_add(4)?)?.try_into().ok()?;
    *at += 4;
    Some(u32::from_le_bytes(bytes))
}
pub fn bytes<'a>(req: &'a [u8], at: &mut usize) -> Option<&'a [u8]> {
    let len = u32_at(req, at)? as usize;
    let end = (*at).checked_add(len)?;
    let value = req.get(*at..end)?;
    *at = end;
    Some(value)
}
pub fn reply(code: i32, data: &[u8]) -> Vec<u8> {
    let mut result = code.to_le_bytes().to_vec();
    if code >= 0 {
        result.extend_from_slice(data);
    }
    result
}
