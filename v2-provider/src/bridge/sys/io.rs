use super::{file, proto};
use crate::bridge::{Callback, InvokeError};

pub fn read_all<I: Callback>(i: &I, name: &str) -> Result<Option<Vec<u8>>, InvokeError> {
    let fd = match file::open_status(i, name, 1)? {
        proto::MISSING => return Ok(None),
        n if n < 0 => return Err(InvokeError::Unavailable),
        fd => fd,
    };
    let mut out = Vec::new();
    let res = loop {
        let (st, bytes) = match file::read(i, fd, 65536) {
            Ok(value) => value,
            Err(_) => break Err(InvokeError::Unavailable),
        };
        if st < 0 {
            break Err(InvokeError::Unavailable);
        }
        if st == 0 {
            break Ok(out);
        }
        if bytes.len() != st as usize {
            break Err(InvokeError::Unavailable);
        }
        if out
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > 8 * 1024 * 1024)
        {
            break Err(InvokeError::Unavailable);
        }
        out.extend_from_slice(&bytes);
    };
    let closed = file::close(i, fd);
    if closed != Ok(0) {
        return Err(InvokeError::Unavailable);
    }
    res.map(Some)
}
pub fn write_all_fd<I: Callback>(i: &I, fd: i32, mut bytes: &[u8]) -> Result<(), InvokeError> {
    while !bytes.is_empty() {
        let n = file::write(i, fd, &bytes[..bytes.len().min(proto::MAX_DATA)])?;
        if n <= 0 || n as usize > bytes.len() {
            return Err(InvokeError::Unavailable);
        }
        bytes = &bytes[n as usize..];
    }
    Ok(())
}
