//! Named read as one VM request: OPEN_DISK -> READ -> CLOSE on the same fd.
//! Descriptor tables live in the per-request VM, so a handle from an earlier
//! HTTP request is already stale; all three frames must share this request.
use super::callback;

const HEAD: usize = 4;
const READ_LIMIT: u32 = 4096;

fn code(reply: &[u8]) -> Option<i32> {
    if reply.len() < HEAD {
        return None;
    }
    let bytes: [u8; HEAD] = reply[..HEAD].try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}

fn pass(reply: &[u8], out: &mut [u8], n: i32) -> i32 {
    let n = n.min(out.len() as i32).max(0);
    out[..n as usize].copy_from_slice(&reply[..n as usize]);
    n
}

/// Execute a framed OPEN/READ/CLOSE chain; return the READ reply verbatim.
/// A failing OPEN, READ or CLOSE is returned as an error code with no data.
pub fn read(req: &[u8], out: &mut [u8]) -> i32 {
    let mut scratch = [0u8; 4096];
    let n = callback::call(req, &mut scratch);
    if n < 0 {
        return n;
    }
    let Some(fd) = code(&scratch[..n as usize]) else {
        return -99;
    };
    if fd < 0 {
        return pass(&scratch[..n as usize], out, n);
    }
    let mut frame = [0u8; 9];
    frame[0] = 2;
    frame[1..5].copy_from_slice(&(fd as u32).to_le_bytes());
    frame[5..9].copy_from_slice(&READ_LIMIT.to_le_bytes());
    let n = callback::call(&frame, out);
    if n < 0 {
        return n;
    }
    let mut close = [0u8; 5];
    close[0] = 4;
    close[1..5].copy_from_slice(&(fd as u32).to_le_bytes());
    let c = callback::call(&close, &mut scratch);
    if c < 0 || code(&scratch[..c as usize]) != Some(0) {
        return c.min(0);
    }
    n
}
