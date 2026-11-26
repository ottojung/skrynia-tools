//! RAM fakes: deterministic random bytes and single-byte CAS.
use super::{Kind, S};

pub unsafe fn secure_random_bits(dst: u64, bits: u64) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let at = dst as usize;
        let n = (bits / 8) as usize;
        if at.saturating_add(n) > b.ram.len() {
            return -6;
        }
        for i in 0..n {
            b.ram[at + i] = 0xA5;
        }
        0
    })
}

pub unsafe fn cas1(fd: i32, offset: u64, expected: i32, desired: i32) -> i32 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        if !matches!(b.get(fd), Some(Kind::Ram)) {
            return -2;
        }
        let (Ok(e), Ok(d), Some(cell)) = (
            u8::try_from(expected),
            u8::try_from(desired),
            b.ram.get_mut(offset as usize),
        ) else {
            return -3;
        };
        let seen = *cell;
        if seen == e {
            *cell = d;
        }
        i32::from(seen)
    })
}
