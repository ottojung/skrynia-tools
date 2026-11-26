//! Descriptor byte streams for the target-ABI P0 child.
//! GET reads the request; replies go back through a framed WRITE operation
//! rather than through a linear-memory buffer or a legacy `write` import.

use super::callback;

const WRITE: u8 = 3;
const OUTPUT: u32 = 1;
const FRAME_HEAD: usize = 9;

pub fn receive(buf: &mut [u8]) -> Result<usize, i32> {
    let mut len = 0usize;
    loop {
        match callback::get() {
            -1 => return Ok(len),
            code if code < 0 => return Err(code as i32),
            code if len >= buf.len() => return Err(-6),
            code => {
                buf[len] = code as u8;
                len += 1;
            }
        }
    }
}

/// Send one framed WRITE to the output descriptor and return its status.
pub fn send(buf: &mut [u8], bytes: &[u8]) -> i32 {
    let total = FRAME_HEAD.saturating_add(bytes.len());
    if total > buf.len() {
        return -6;
    }
    buf[0] = WRITE;
    buf[1..5].copy_from_slice(&OUTPUT.to_le_bytes());
    buf[5..9].copy_from_slice(&(bytes.len() as u32).to_le_bytes());
    buf[FRAME_HEAD..total].copy_from_slice(bytes);
    let mut reply = [0u8; 4];
    let n = callback::call(&buf[..total], &mut reply);
    if n != 4 {
        return if n < 0 { n } else { -5 };
    }
    i32::from_le_bytes(reply)
}
