pub const OPEN_DISK: u8 = 1;
pub const READ: u8 = 2;
pub const WRITE: u8 = 3;
pub const CLOSE: u8 = 4;
pub const SYNC: u8 = 5;
pub const SEEK: u8 = 6;
pub const TRUNCATE: u8 = 7;
pub const RENAME: u8 = 8;
pub const DELETE: u8 = 9;
pub const MKDIR: u8 = 10;
pub const EXISTS: u8 = 11;
pub const IS_DIR: u8 = 12;
pub const OPEN_DIR: u8 = 13;
pub const NEXT_ENTRY: u8 = 14;
pub const RANDOM: u8 = 15;
pub const OPEN_RAM: u8 = 16;
pub const CAS_RAM: u8 = 17;
pub const LIMIT: usize = 64 * 1024;
pub const INVALID: i32 = -1;
pub const OVERFLOW: i32 = -11;

pub fn u32_at(req: &[u8], at: &mut usize) -> Option<u32> {
    let bytes: [u8; 4] = req.get(*at..(*at).checked_add(4)?)?.try_into().ok()?;
    *at += 4;
    Some(u32::from_le_bytes(bytes))
}
pub fn u64_at(req: &[u8], at: &mut usize) -> Option<u64> {
    let bytes: [u8; 8] = req.get(*at..(*at).checked_add(8)?)?.try_into().ok()?;
    *at += 8;
    Some(u64::from_le_bytes(bytes))
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
