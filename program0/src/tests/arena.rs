//! Private code staging and monotone shared metadata allocation.
use super::{abi, alloc, code, fake};

#[test]
fn code_accepts_exact_capacity_without_shared_staging() {
    for excess in [0, 1] {
        let bytes = vec![0xAA; code::MAX_CODE + excess];
        let path = String::from_utf8_lossy(code::GUEST_PATH).into_owned();
        let (_, _, _) = fake::seed(vec![], vec![(path, bytes)]);
        let result = code::load();
        if excess == 0 {
            assert_eq!(result.unwrap(), vec![0xaa; code::MAX_CODE]);
        } else {
            assert_eq!(result, Err(alloc::ERR_RANGE));
        }
        fake::inspect(|s| {
            assert!(s.ram.is_empty());
            assert!(s.get((s.fds.len() - 1) as i32).is_none());
        });
    }
}

#[test]
fn ensure_never_shrinks_existing_ram() {
    let (_, _, ram) = fake::seed(vec![], vec![]);
    let beyond = alloc::META_END + 50;
    assert_eq!(unsafe { abi::truncate(ram, beyond + 10) }, 0);
    assert_eq!(unsafe { abi::seek(ram, beyond as i64) }, beyond as i64);
    assert_eq!(unsafe { abi::write1(ram, 0x5A) }, 0);
    assert_eq!(unsafe { abi::seek(ram, -(beyond as i64 + 1)) }, 0);
    assert_eq!(alloc::ensure(ram), Ok(()));
    assert_eq!(unsafe { abi::seek(ram, beyond as i64) }, beyond as i64);
    assert_eq!(unsafe { abi::read1(ram) }, 0x5A);
}
