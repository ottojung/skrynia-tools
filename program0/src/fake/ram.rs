//! RAM fakes: deterministic random bytes and single-byte CAS.
use super::{Kind, S};

pub unsafe fn secure_random_bits(dst: u64, bits: u64) -> i32 {
    let n = bits.div_ceil(8) as usize;
    let out = unsafe { core::slice::from_raw_parts_mut(dst as usize as *mut u8, n) };
    out.fill(0xa5);
    if !bits.is_multiple_of(8) {
        out[n - 1] &= (1u8 << (bits % 8)) - 1;
    }
    0
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
