use super::*;
pub fn page(out: &mut Vec<u8>, after: &Option<Vec<u8>>, limit: u16) {
    match after {
        None => out.push(0),
        Some(hash) => {
            out.push(1);
            put_hash(out, hash);
        }
    }
    out.extend_from_slice(&limit.to_le_bytes());
}
