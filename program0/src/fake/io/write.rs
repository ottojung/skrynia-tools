use super::{Kind, S};

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
                    return -7;
                }
                b.ram[p] = v;
                b.rpos = p + 1;
                0
            }
            (Some(Kind::Disk(i)), Ok(v)) => {
                let p = b.fpos[fd as usize];
                if p == b.files[i].1.len() {
                    b.files[i].1.push(v);
                } else {
                    b.files[i].1[p] = v;
                }
                b.fpos[fd as usize] += 1;
                0
            }
            _ => -3,
        }
    })
}
