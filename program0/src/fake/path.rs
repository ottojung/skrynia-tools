//! Private-buffer path lookup fakes.
use super::{Kind, S};

// Test-only ABI calls receive real native pointers to caller-owned slices.
pub(super) unsafe fn name(ptr: u64, len: u64) -> String {
    let bytes = unsafe { core::slice::from_raw_parts(ptr as usize as *const u8, len as usize) };
    String::from_utf8_lossy(bytes).into_owned()
}

pub unsafe fn open_disk(path: u64, len: u64, _mode: i32) -> i32 {
    let name = unsafe { name(path, len) };
    S.with(|s| {
        let mut b = s.borrow_mut();
        match b.files.iter().position(|f| f.0 == name) {
            Some(i) => b.fd(Kind::Disk(i)),
            None => -8,
        }
    })
}

pub unsafe fn exists(path: u64, len: u64) -> i32 {
    let name = unsafe { name(path, len) };
    S.with(|s| i32::from(s.borrow().files.iter().any(|f| f.0 == name)))
}
