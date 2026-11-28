//! Deterministic NEXT_ENTRY fake writing to a private buffer.
use super::{Kind, S};

fn parent(p: &str) -> &str {
    p.rsplit_once('/').map(|x| x.0).unwrap_or("")
}

pub unsafe fn next_entry(fd: i32, dst: u64, cap: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let Some(Kind::Dir(i)) = b.get(fd).cloned() else {
            return -2;
        };
        let dir = b.dirs[i].clone();
        let mut names: Vec<String> = b
            .files
            .iter()
            .filter(|f| parent(&f.0) == dir)
            .map(|f| f.0.rsplit('/').next().unwrap_or("").to_string())
            .collect();
        names.extend(
            b.dirs
                .iter()
                .filter(|d| parent(d) == dir)
                .map(|d| d.rsplit('/').next().unwrap_or("").to_string()),
        );
        let at = b.fpos[fd as usize];
        let Some(entry) = names.get(at) else {
            return 0;
        };
        if cap < entry.len() as u64 {
            return -7;
        }
        unsafe {
            core::ptr::copy_nonoverlapping(entry.as_ptr(), dst as usize as *mut u8, entry.len())
        };
        b.fpos[fd as usize] += 1;
        entry.len() as i32
    })
}
