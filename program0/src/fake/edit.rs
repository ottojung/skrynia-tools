//! In-memory fakes for DELETE and RENAME descriptor ops.
use super::{S, path};

pub unsafe fn delete(path: u64, len: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let n = unsafe { path::name(path, len) };
        if let Some(i) = b.files.iter().position(|f| f.0 == n) {
            b.files.remove(i);
            return 0;
        }
        if let Some(i) = b.dirs.iter().position(|d| *d == n) {
            b.dirs.remove(i);
            return 0;
        }
        -2
    })
}

pub unsafe fn rename(a: u64, al: u64, b: u64, bl: u64) -> i32 {
    S.with(|s| {
        let mut st = s.borrow_mut();
        let (src, dst) = unsafe { (path::name(a, al), path::name(b, bl)) };
        if let Some(i) = st.files.iter().position(|f| f.0 == src) {
            st.files[i].0 = dst;
            return 0;
        }
        if let Some(i) = st.dirs.iter().position(|d| *d == src) {
            st.dirs[i] = dst;
            return 0;
        }
        -2
    })
}
