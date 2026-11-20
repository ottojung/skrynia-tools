use super::{Callback, InvokeError, file, io, mkdir_p, path, rand, sync_dir};
pub fn write_atomic<I: Callback>(i: &I, name: &str, bytes: &[u8]) -> Result<(), InvokeError> {
    let (parent, basename) = name.rsplit_once('/').ok_or(InvokeError::Unavailable)?;
    mkdir_p(i, parent)?;
    let rnd = rand::bytes(i, 8)?;
    let tmp = format!(
        "{parent}/.{basename}.tmp.{:016x}",
        u64::from_le_bytes(rnd.try_into().map_err(|_| InvokeError::Unavailable)?)
    );
    let fd = file::open_disk(i, &tmp, 2 + 4 + 8 + 16)?;
    let saved = io::write_all_fd(i, fd, bytes).and_then(|_| {
        if path::sync(i, fd)? == 0 {
            Ok(())
        } else {
            Err(InvokeError::Unavailable)
        }
    });
    let closed = file::close(i, fd);
    if saved.is_err() || closed != Ok(0) {
        let _ = path::delete(i, &tmp);
        let _ = sync_dir(i, parent);
        return Err(InvokeError::Unavailable);
    }
    if path::rename(i, &tmp, name) != Ok(0) {
        let _ = path::delete(i, &tmp);
        let _ = sync_dir(i, parent);
        return Err(InvokeError::Unavailable);
    }
    sync_dir(i, parent)
}

#[cfg(test)]
mod tests {
    use super::write_atomic;
    use crate::bridge::{Callback, InvokeError};
    use std::cell::RefCell;

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Act {
        Mkdir(String),
        Rand,
        Open(String),
        Write(i32),
        Sync(i32),
        Close(i32),
        Rename(String, String),
        Delete(String),
        DirOpen(String),
        DirSync(i32),
        DirClose(i32),
        Exists(String),
        IsDir(String),
    }

    struct Fake {
        log: RefCell<Vec<Act>>,
        rename: RefCell<Result<i32, InvokeError>>,
        write: RefCell<Result<i32, InvokeError>>,
    }
    impl Fake {
        fn new(rename: Result<i32, InvokeError>, write: Result<i32, InvokeError>) -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                rename: RefCell::new(rename),
                write: RefCell::new(write),
            }
        }
    }
    fn take_str(tail: &[u8]) -> (String, &[u8]) {
        let n = u32::from_le_bytes(tail[..4].try_into().unwrap()) as usize;
        (
            String::from_utf8(tail[4..4 + n].to_vec()).unwrap(),
            &tail[4 + n..],
        )
    }
    impl Callback for Fake {
        fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, InvokeError> {
            let op = req[0];
            match op {
                1 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::Open(path));
                    Ok(20i32.to_le_bytes().to_vec())
                }
                3 => {
                    let fd = i32::from_le_bytes(req[1..5].try_into().unwrap());
                    self.log.borrow_mut().push(Act::Write(fd));
                    let n = self.write.borrow_mut().clone()?;
                    Ok(n.to_le_bytes().to_vec())
                }
                4 => {
                    let fd = i32::from_le_bytes(req[1..5].try_into().unwrap());
                    let log = self.log.borrow().clone();
                    if matches!(log.last(), Some(Act::DirOpen(_)) | Some(Act::DirSync(_))) {
                        self.log.borrow_mut().push(Act::DirClose(fd));
                    } else {
                        self.log.borrow_mut().push(Act::Close(fd));
                    }
                    Ok(0i32.to_le_bytes().to_vec())
                }
                5 => {
                    let fd = i32::from_le_bytes(req[1..5].try_into().unwrap());
                    let in_dir = self
                        .log
                        .borrow()
                        .iter()
                        .rev()
                        .find_map(|a| match a {
                            Act::DirOpen(_) => Some(true),
                            Act::Open(_) => Some(false),
                            _ => None,
                        })
                        .unwrap_or(false);
                    if in_dir {
                        self.log.borrow_mut().push(Act::DirSync(fd));
                    } else {
                        self.log.borrow_mut().push(Act::Sync(fd));
                    }
                    Ok(0i32.to_le_bytes().to_vec())
                }
                6 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::Mkdir(path));
                    Ok(0i32.to_le_bytes().to_vec())
                }
                7 => {
                    let (a, rest) = take_str(&req[1..]);
                    let (b, _) = take_str(rest);
                    self.log.borrow_mut().push(Act::Rename(a, b));
                    let r = self.rename.borrow_mut().clone()?;
                    Ok(r.to_le_bytes().to_vec())
                }
                8 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::Delete(path));
                    Ok(0i32.to_le_bytes().to_vec())
                }
                9 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::DirOpen(path));
                    Ok(30i32.to_le_bytes().to_vec())
                }
                11 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::Exists(path));
                    Ok(1i32.to_le_bytes().to_vec())
                }
                12 => {
                    self.log.borrow_mut().push(Act::Rand);
                    let mut out = 8i32.to_le_bytes().to_vec();
                    out.extend_from_slice(&1u64.to_le_bytes());
                    Ok(out)
                }
                16 => {
                    let (path, _) = take_str(&req[1..]);
                    self.log.borrow_mut().push(Act::IsDir(path));
                    Ok(1i32.to_le_bytes().to_vec())
                }
                _ => Err(InvokeError::Unavailable),
            }
        }
    }

    #[test]
    fn rename_transport_failure_cleans_staging_and_syncs_parent() {
        let f = Fake::new(Err(InvokeError::Unavailable), Ok(2));
        assert!(write_atomic(&f, "v2/o", b"hi").is_err());
        let log = f.log.borrow().clone();
        let tmp = "v2/.o.tmp.0000000000000001".to_string();
        assert!(log.contains(&Act::Rename(tmp.clone(), "v2/o".into())));
        assert!(log.contains(&Act::Delete(tmp)));
        assert!(
            log.iter()
                .any(|a| matches!(a, Act::DirOpen(p) if p == "v2"))
        );
    }
    #[test]
    fn rename_negative_status_cleans_staging_and_syncs_parent() {
        let f = Fake::new(Ok(-1), Ok(2));
        assert!(write_atomic(&f, "v2/o", b"hi").is_err());
        let log = f.log.borrow().clone();
        let tmp = "v2/.o.tmp.0000000000000001".to_string();
        assert!(log.contains(&Act::Delete(tmp)));
    }
    #[test]
    fn write_failure_cleans_staging_and_syncs_parent() {
        let f = Fake::new(Ok(0), Err(InvokeError::Unavailable));
        assert!(write_atomic(&f, "v2/o", b"hi").is_err());
        let log = f.log.borrow().clone();
        let tmp = "v2/.o.tmp.0000000000000001".to_string();
        assert!(log.contains(&Act::Delete(tmp)));
        assert!(!log.iter().any(|a| matches!(a, Act::Rename(_, _))));
    }
}
