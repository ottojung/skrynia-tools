//! Complete opaque callback frames over the single supplied `p0.invoke`.
use crate::{CB_FLUSH, CB_GET, CB_PUT, byte, call};

/// Append one byte to the pending P1-owned request frame.
fn put(value: u8) -> i32 {
    if call(CB_PUT, i32::from(value)) < 0 {
        return -8;
    }
    0
}

/// Run one complete request exactly once and drain its opaque reply.
pub fn invoke_raw(req: &[u8], out: &mut [u8]) -> i32 {
    for &value in req {
        if put(value) != 0 {
            return -8;
        }
    }
    if call(CB_FLUSH, 0) != 0 {
        return -8;
    }
    let mut len = 0;
    loop {
        let value = call(CB_GET, 0);
        if value == -1 {
            break;
        }
        let Ok(decoded) = byte(value) else {
            return -8;
        };
        if len >= out.len() {
            return -8;
        }
        out[len] = decoded;
        len += 1;
    }
    len as i32
}
