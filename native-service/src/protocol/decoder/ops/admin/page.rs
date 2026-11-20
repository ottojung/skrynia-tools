use super::{DecodeErr, hash, take};
pub fn page(src: &mut &[u8]) -> Result<(Option<Vec<u8>>, u16), DecodeErr> {
    let after = match take(src, 1)?[0] {
        0 => None,
        1 => Some(hash(src)?),
        _ => return Err(DecodeErr::Op),
    };
    let n: [u8; 2] = take(src, 2)?.try_into().unwrap();
    let limit = u16::from_le_bytes(n);
    if limit == 0 || limit > 256 {
        return Err(DecodeErr::Op);
    }
    Ok((after, limit))
}
