//! Byte-stream fakes: read1 and write1.
use super::{Kind, S};

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

pub unsafe fn write1(fd: i32, byte: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        match (b.get(fd).cloned(), u8::try_from(byte)) {
            (Some(Kind::Output), Ok(v)) => {
                b.output.push(v);
                0
            }
            (Some(Kind::Ram), Ok(v)) => {
                let p = b.rpos;
                if p >= b.ram.len() {
                    b.ram.resize(p + 1, 0)
                }
                b.ram[p] = v;
                b.rpos = p + 1;
                0
            }
            _ => -3,
        }
    })
}
