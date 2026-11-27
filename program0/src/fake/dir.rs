//! In-memory fakes for mkdir/open_directory/sync.
use super::{Kind, S, path};

pub unsafe fn mkdir(path: u64, len: u64, _mode: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let n = unsafe { path::name(path, len) };
        if b.dirs.contains(&n) || b.files.iter().any(|f| f.0 == n) {
            return -3;
        }
        b.dirs.push(n);
        0
    })
}

pub unsafe fn open_directory(path: u64, len: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let n = unsafe { path::name(path, len) };
        match b.dirs.iter().position(|d| *d == n) {
            Some(i) => b.fd(Kind::Dir(i)),
            None => -8,
        }
    })
}

pub unsafe fn sync(_fd: i32) -> i32 {
    0
}
