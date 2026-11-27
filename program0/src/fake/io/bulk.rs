use super::{Kind, S, read1, write1};

pub unsafe fn read(fd: i32, dst: u64, len: u64) -> i64 {
    if !S.with(|s| {
        matches!(
            s.borrow().get(fd),
            Some(Kind::Input | Kind::Ram | Kind::Disk(_))
        )
    }) {
        return -2;
    }
    let out = unsafe { core::slice::from_raw_parts_mut(dst as usize as *mut u8, len as usize) };
    for (i, byte) in out.iter_mut().enumerate() {
        match unsafe { read1(fd) } {
            -1 => return i as i64,
            e if e < 0 => return if i == 0 { i64::from(e) } else { i as i64 },
            b => *byte = b as u8,
        }
    }
    len as i64
}

pub unsafe fn write(fd: i32, src: u64, len: u64) -> i64 {
    if !S.with(|s| {
        matches!(
            s.borrow().get(fd),
            Some(Kind::Output | Kind::Ram | Kind::Disk(_))
        )
    }) {
        return -2;
    }
    let bytes = unsafe { core::slice::from_raw_parts(src as usize as *const u8, len as usize) };
    for (i, byte) in bytes.iter().enumerate() {
        let rc = unsafe { write1(fd, i32::from(*byte)) };
        if rc != 0 {
            return if i == 0 { i64::from(rc) } else { i as i64 };
        }
    }
    len as i64
}
