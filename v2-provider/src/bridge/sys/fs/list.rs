use super::{Callback, InvokeError, dir, file};
pub fn list_dir<I: Callback>(i: &I, name: &str) -> Result<Vec<String>, InvokeError> {
    let fd = dir::open(i, name)?;
    let result = (|| {
        let mut out = Vec::new();
        loop {
            let (st, data) = dir::next(i, fd)?;
            if st < 0 {
                return Err(InvokeError::Unavailable);
            }
            if st == 0 {
                return Ok(out);
            }
            if st as usize != data.len() {
                return Err(InvokeError::Unavailable);
            }
            out.push(String::from_utf8(data).map_err(|_| InvokeError::Unavailable)?);
        }
    })();
    let closed = file::close(i, fd);
    if closed != Ok(0) {
        return Err(InvokeError::Unavailable);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::list_dir;
    use crate::bridge::{Callback, InvokeError};
    use std::cell::RefCell;

    struct Fake {
        log: RefCell<Vec<String>>,
        next: RefCell<Vec<Result<(i32, Vec<u8>), InvokeError>>>,
        close_ok: bool,
    }
    impl Fake {
        fn new(next: Vec<Result<(i32, Vec<u8>), InvokeError>>, close_ok: bool) -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                next: RefCell::new(next),
                close_ok,
            }
        }
    }
    impl Callback for Fake {
        fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, InvokeError> {
            match req.first().copied() {
                Some(9) => {
                    self.log.borrow_mut().push("open".into());
                    Ok(11i32.to_le_bytes().to_vec())
                }
                Some(10) => {
                    self.log.borrow_mut().push("next".into());
                    let next = self.next.borrow_mut().remove(0);
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
    fn next_transport_error_closes_dir_fd() {
        let f = Fake::new(vec![Err(InvokeError::Unavailable)], true);
        assert!(list_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "next", "close"]);
    }
    #[test]
    fn next_negative_status_closes_dir_fd() {
        let f = Fake::new(vec![Ok((-1, vec![]))], true);
        assert!(list_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "next", "close"]);
    }
    #[test]
    fn malformed_dir_entry_length_closes_dir_fd() {
        let f = Fake::new(vec![Ok((3, b"ab".to_vec()))], true);
        assert!(list_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "next", "close"]);
    }
    #[test]
    fn close_failure_is_reported() {
        let f = Fake::new(vec![Ok((0, vec![]))], false);
        assert!(list_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "next", "close"]);
    }
}
