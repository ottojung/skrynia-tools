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

#[cfg(test)]
mod tests {
    use super::read_all;
    use crate::bridge::{Callback, InvokeError};
    use std::cell::RefCell;

    struct Fake {
        log: RefCell<Vec<String>>,
        read: RefCell<Vec<Result<(i32, Vec<u8>), InvokeError>>>,
        close_ok: bool,
    }
    impl Fake {
        fn new(read: Vec<Result<(i32, Vec<u8>), InvokeError>>, close_ok: bool) -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                read: RefCell::new(read),
                close_ok,
            }
        }
    }
    impl Callback for Fake {
        fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, InvokeError> {
            match req.first().copied() {
                Some(1) => {
                    self.log.borrow_mut().push("open".into());
                    Ok(7i32.to_le_bytes().to_vec())
                }
                Some(2) => {
                    self.log.borrow_mut().push("read".into());
                    let next = self.read.borrow_mut().remove(0);
                    let (st, data) = next?;
                    let mut out = st.to_le_bytes().to_vec();
                    out.extend_from_slice(&data);
                    Ok(out)
                }
                Some(4) => {
                    self.log.borrow_mut().push("close".into());
                    Ok(if self.close_ok { 0i32 } else { -1i32 }
                        .to_le_bytes()
                        .to_vec())
                }
                _ => Err(InvokeError::Unavailable),
            }
        }
    }

    fn log(f: &Fake) -> Vec<String> {
        f.log.borrow().clone()
    }

    #[test]
    fn read_transport_error_closes_fd() {
        let f = Fake::new(vec![Err(InvokeError::Unavailable)], true);
        assert!(read_all(&f, "x").is_err());
        assert_eq!(log(&f), ["open", "read", "close"]);
    }
    #[test]
    fn read_negative_status_closes_fd() {
        let f = Fake::new(vec![Ok((-1, vec![]))], true);
        assert!(read_all(&f, "x").is_err());
        assert_eq!(log(&f), ["open", "read", "close"]);
    }
    #[test]
    fn read_malformed_length_closes_fd() {
        let f = Fake::new(vec![Ok((2, vec![1]))], true);
        assert!(read_all(&f, "x").is_err());
        assert_eq!(log(&f), ["open", "read", "close"]);
    }
    #[test]
    fn close_failure_is_reported_after_successful_read() {
        let f = Fake::new(vec![Ok((0, vec![]))], false);
        assert!(read_all(&f, "x").is_err());
        assert_eq!(log(&f), ["open", "read", "close"]);
    }
}
