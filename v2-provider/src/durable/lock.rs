use crate::bridge::sys::{file, ram};
use crate::bridge::Callback;
pub struct Guard { fd: i32 }
pub fn acquire<I: Callback>(i: &I) -> Option<Guard> {
    let fd = ram::open(i).ok()?;
    // Spin-limited cross-request lock via shared RAM offset 0.
    for _ in 0..10000 {
        if let Ok((rc, old)) = ram::cas_u64(i, fd, 0, 0, 1) {
            if rc == 0 && old == 0 { return Some(Guard { fd }); }
        } else { break; }
    }
    let _ = file::close(i, fd);
    None
}
pub fn release<I: Callback>(i: &I, g: Guard) {
    let _ = ram::cas_u64(i, g.fd, 0, 1, 0);
    let _ = file::close(i, g.fd);
}
