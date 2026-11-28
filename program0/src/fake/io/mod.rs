//! Byte-stream fakes: read1 and write1.
use super::{Kind, S};
mod bulk;
mod write;
pub use bulk::*;
pub use write::*;

pub unsafe fn read1(fd: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        match b.get(fd).cloned() {
            Some(Kind::Input) => {
                let v = b.input.get(b.inp).copied();
                b.inp += usize::from(v.is_some());
                v.map(i32::from).unwrap_or(-1)
            }
            Some(Kind::Disk(i)) => {
                let v = b.files[i].1.get(b.fpos[fd as usize]).copied();
                b.fpos[fd as usize] += usize::from(v.is_some());
                v.map(i32::from).unwrap_or(-1)
            }
            Some(Kind::Ram) => {
                let v = b.ram.get(b.rpos).copied();
                b.rpos += usize::from(v.is_some());
                v.map(i32::from).unwrap_or(-1)
            }
            _ => -2,
        }
    })
}
