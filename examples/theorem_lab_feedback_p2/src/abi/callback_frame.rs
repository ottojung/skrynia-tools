//! P1-owned generic callback framing. CB_PUT appends one opaque request
//! byte, CB_FLUSH executes exactly one complete request through the callback,
//! and CB_GET drains one reply byte. Sticky errors prevent replay.
use super::inner::{call, flush, CB_FLUSH, CB_GET, CB_PUT};

pub fn invoke_raw(req: &[u8], out: &mut [u8]) -> i32 {
    for &byte in req {
        if call(CB_PUT, i32::from(byte)).is_err() {
            return -3;
        }
    }
    if !flush(CB_FLUSH) {
        return -5;
    }
    let mut len: usize = 0;
    loop {
        match call(CB_GET, 0) {
            Err(-1) => return len as i32,
            Err(code) => return code,
            Ok(byte) if len >= out.len() => return -6,
            Ok(byte) => {
                out[len] = byte;
                len += 1;
            }
        }
    }
}
