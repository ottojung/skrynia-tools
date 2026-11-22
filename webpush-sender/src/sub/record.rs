//! SubFetch frame: u32 count, then repeated u32 length + subscription bytes.
use super::{Error, MAX_SUBS, Result, Subscription};

pub fn parse_frame(src: &[u8]) -> Result<Vec<Subscription>> {
    if src.len() < 4 {
        return Err(Error::Wire("frame"));
    }
    let count = u32::from_le_bytes(src[..4].try_into().expect("4")) as usize;
    if count > MAX_SUBS {
        return Err(Error::Quota);
    }
    let mut at = 4;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        if at + 4 > src.len() {
            return Err(Error::Wire("frame-len"));
        }
        let len = u32::from_le_bytes(src[at..at + 4].try_into().expect("4")) as usize;
        at += 4;
        if len == 0 || len > 4096 || at + len > src.len() {
            return Err(Error::Wire("frame-body"));
        }
        out.push(Subscription::parse(&src[at..at + len])?);
        at += len;
    }
    if at != src.len() {
        return Err(Error::Wire("frame-tail"));
    }
    Ok(out)
}
