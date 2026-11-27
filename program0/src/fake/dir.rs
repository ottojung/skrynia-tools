//! In-memory fakes for mkdir/open_directory/sync.
use super::{Kind, S, State};

fn name(b: &State, p: u64, n: u64) -> String {
    let at = (p as usize).min(b.ram.len());
    let end = at.saturating_add(n as usize).min(b.ram.len());
    String::from_utf8_lossy(&b.ram[at..end]).into_owned()
}

pub unsafe fn mkdir(path: u64, len: u64, _mode: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let n = name(&b, path, len);
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
        let n = name(&b, path, len);
        match b.dirs.iter().position(|d| *d == n) {
            Some(i) => b.fd(Kind::Dir(i)),
            None => -8,
        }
    })
}

pub unsafe fn sync(_fd: i32) -> i32 {
    0
}
