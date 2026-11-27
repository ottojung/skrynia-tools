//! Relative seek and recorded eval calls for host tests.
use super::{Kind, S};

pub unsafe fn seek(fd: i32, shift: i64) -> i64 {
    S.with(|s| {
        let mut b = s.borrow_mut();
        let k = b.get(fd).cloned();
        let (pos, len) = match k {
            Some(Kind::Ram) => (b.rpos, b.ram.len()),
            Some(Kind::Input) => (b.inp, b.input.len()),
            Some(Kind::Disk(i)) => (b.fpos[fd as usize], b.files[i].1.len()),
            _ => return -2,
        };
        let next = (pos as i64).checked_add(shift).unwrap_or(i64::MAX);
        if next < 0 {
            return -6;
        }
        let next = usize::try_from(next).unwrap_or(usize::MAX);
        let next = match k {
            Some(Kind::Input) if shift >= 0 => next.min(len),
            Some(Kind::Input) => return -4,
            _ if next > len => return -6,
            _ => next,
        };
        match k {
            Some(Kind::Ram) => b.rpos = next,
            Some(Kind::Input) => b.inp = next,
            Some(Kind::Disk(_)) => b.fpos[fd as usize] = next,
            _ => {}
        }
        next as i64
    })
}

/// Force the next recorded eval to trap, without executing guest bytes.
pub fn set_eval_result(rc: i64) {
    S.with(|s| s.borrow_mut().eval_result = rc);
}

pub unsafe fn eval(code: u64, size: u64, _f: i32, _a: i32, _b: i32) -> i64 {
    S.with(|s| {
        let mut v = s.borrow_mut();
        v.eval_calls += 1;
        v.eval_code =
            unsafe { core::slice::from_raw_parts(code as usize as *const u8, size as usize) }
                .to_vec();
        v.eval_result
    })
}
