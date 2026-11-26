//! In-memory fakes for DELETE and RENAME descriptor ops.
use super::{S, State};

fn name(b: &State, p: u64, n: u64) -> String {
    let at = (p as usize).min(b.ram.len());
    let end = at.saturating_add(n as usize).min(b.ram.len());
    String::from_utf8_lossy(&b.ram[at..end]).into_owned()
}

pub unsafe fn delete(path: u64, len: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let n = name(&b, path, len);
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
        let (src, dst) = (name(&st, a, al), name(&st, b, bl));
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
