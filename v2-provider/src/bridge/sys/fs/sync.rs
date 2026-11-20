use super::{Callback, InvokeError, dir, file, path};
pub fn sync_dir<I: Callback>(i: &I, name: &str) -> Result<(), InvokeError> {
    let fd = dir::open(i, name)?;
    let sync_result = path::sync(i, fd);
    let closed = file::close(i, fd);
    if sync_result != Ok(0) || closed != Ok(0) {
        return Err(InvokeError::Unavailable);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::sync_dir;
    use crate::bridge::{Callback, InvokeError};
    use std::cell::RefCell;

    struct Fake {
        log: RefCell<Vec<String>>,
        sync: Result<i32, InvokeError>,
        close_ok: bool,
    }
    impl Fake {
        fn new(sync: Result<i32, InvokeError>, close_ok: bool) -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                sync,
                close_ok,
            }
        }
    }
    impl Callback for Fake {
        fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, InvokeError> {
            match req.first().copied() {
                Some(9) => {
                    self.log.borrow_mut().push("open".into());
                    Ok(5i32.to_le_bytes().to_vec())
                }
                Some(5) => {
                    self.log.borrow_mut().push("sync".into());
                    self.sync.clone().map(|s| s.to_le_bytes().to_vec())
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
    fn sync_error_still_closes_fd() {
        let f = Fake::new(Err(InvokeError::Unavailable), true);
        assert!(sync_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "sync", "close"]);
    }
    #[test]
    fn sync_negative_status_still_closes_fd() {
        let f = Fake::new(Ok(-1), true);
        assert!(sync_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "sync", "close"]);
    }
    #[test]
    fn close_failure_is_reported() {
        let f = Fake::new(Ok(0), false);
        assert!(sync_dir(&f, "d").is_err());
        assert_eq!(log(&f), ["open", "sync", "close"]);
    }
}
