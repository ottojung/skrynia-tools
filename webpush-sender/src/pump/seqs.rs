//! Hint sequence list: u32 count then sorted u64 LE seqs.
use super::{Error, MAX_HINTS, Result};

pub fn parse(src: &[u8]) -> Result<Vec<u64>> {
    if src.len() < 4 {
        return Err(Error::Wire("list"));
    }
    let count = u32::from_le_bytes(src[..4].try_into().expect("4")) as usize;
    if count > MAX_HINTS || src.len() != 4 + 8 * count {
        return Err(Error::Wire("list-len"));
    }
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let at = 4 + 8 * i;
        let seq = u64::from_le_bytes(src[at..at + 8].try_into().expect("8"));
        out.push(seq);
    }
    if out.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(Error::Wire("list-order"));
    }
    Ok(out)
}
