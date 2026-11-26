//! RAM-staged path lookup fakes.
use super::{Kind, S};

pub unsafe fn open_disk(path: u64, len: u64, _mode: i32) -> i32 {
    S.with(|s| {
        let b = s.borrow();
        let name =
            String::from_utf8_lossy(&b.ram[path as usize..(path + len) as usize]).into_owned();
        drop(b);
        let mut b = s.borrow_mut();
        match b.files.iter().position(|f| f.0 == name) {
            Some(i) => b.fd(Kind::Disk(i)),
            None => -8,
        }
    })
}

pub unsafe fn exists(path: u64, len: u64) -> i32 {
    S.with(|s| {
        let b = s.borrow();
        let name = String::from_utf8_lossy(&b.ram[path as usize..(path + len) as usize]);
        i32::from(b.files.iter().any(|f| f.0 == name))
    })
}
