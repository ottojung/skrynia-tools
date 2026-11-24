//! Fixed-size durable slot encoding. Unknown or corrupt bytes fail closed.
use super::PushErr;

const MAGIC: &[u8; 4] = b"WSL1";
const LEN: usize = 16;

pub(crate) fn encode(generation: u64, dirty: bool) -> [u8; LEN] {
    let mut b = [0u8; LEN];
    b[..4].copy_from_slice(MAGIC);
    b[4..12].copy_from_slice(&generation.to_le_bytes());
    b[12] = u8::from(dirty);
    b
}

pub(crate) fn decode<D, L>(raw: Option<Vec<u8>>) -> Result<(u64, bool), PushErr<D, L>> {
    match raw {
        None => Ok((0, false)),
        Some(b)
            if b.len() == LEN
                && &b[..4] == MAGIC
                && b[12] <= 1
                && b[13..].iter().all(|x| *x == 0) =>
        {
            Ok((u64::from_le_bytes(b[4..12].try_into().unwrap()), b[12] == 1))
        }
        Some(_) => Err(PushErr::BadKey),
    }
}
