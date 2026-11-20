pub const DISK_READ: u8 = 1;
pub const DISK_WRITE_ATOMIC: u8 = 2;
pub const DISK_REMOVE: u8 = 3;
pub const DISK_LIST: u8 = 4;
pub const LOCK_ACQUIRE: u8 = 5;
pub const LOCK_RELEASE: u8 = 6;
pub const RANDOM_FILL: u8 = 7;
pub const EVAL_APP: u8 = 8;
pub const OK: u8 = 0;
pub const ERR: u8 = 1;
pub const NONE: u8 = 2;
pub const SOME: u8 = 3;
pub const FALSE: u8 = 0;
pub const TRUE: u8 = 1;
pub fn put_u32(out: &mut Vec<u8>, n: u32) {
    out.extend_from_slice(&n.to_le_bytes());
}
pub fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_u32(out, b.len() as u32);
    out.extend_from_slice(b);
}
pub fn get_u32(src: &[u8], i: &mut usize) -> Option<u32> {
    let b = src.get(*i..*i + 4)?;
    *i += 4;
    Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}
pub fn get_bytes(src: &[u8], i: &mut usize) -> Option<Vec<u8>> {
    let n = get_u32(src, i)? as usize;
    let b = src.get(*i..*i + n)?;
    *i += n;
    Some(b.to_vec())
}
pub fn get_str(src: &[u8], i: &mut usize) -> Option<String> {
    String::from_utf8(get_bytes(src, i)?).ok()
}
pub fn ok_empty() -> Vec<u8> {
    vec![OK]
}
pub fn err(code: i32) -> Vec<u8> {
    let mut out = vec![ERR];
    put_u32(&mut out, code as u32);
    out
}
