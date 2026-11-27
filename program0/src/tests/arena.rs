//! Regression tests for staging bounds and monotone RAM growth.
use super::{abi, alloc, code, fake};

#[test]
fn stage_accepts_exact_capacity_but_never_crosses_slot_boundary() {
    let cap = alloc::SLOT_SIZE - alloc::SLOT_CODE_AT;
    for excess in [0, 1] {
        let bytes = vec![0xAA; (cap + excess) as usize];
        let path = String::from_utf8_lossy(code::GUEST_PATH).into_owned();
        let (_, _, ram) = fake::seed(vec![], vec![(path, bytes)]);
        alloc::ensure(ram).expect("arena");
        let (_, base) = alloc::acquire(ram).expect("slot");
        let end = base + alloc::SLOT_SIZE;
        fake::S.with(|s| s.borrow_mut().ram[end as usize] = 0x5A);
        let mut pos = 0;
        code::put_path(ram, &mut pos, base, code::GUEST_PATH).unwrap();
        let result = code::stage(
            ram,
            &mut pos,
            base + alloc::SLOT_CODE_AT,
            base + alloc::SLOT_PATH_AT,
        );
        let expected = [Ok(cap), Err(alloc::ERR_RANGE)][excess as usize];
        assert_eq!(result, expected);
        assert_eq!(pos, end);
        fake::inspect(|s| {
            assert_eq!(s.ram[end as usize - 1], 0xAA);
            assert_eq!(s.ram[end as usize], 0x5A);
            assert!(s.get((s.fds.len() - 1) as i32).is_none());
        });
    }
}

#[test]
fn ensure_never_shrinks_existing_ram() {
    let (_, _, ram) = fake::seed(vec![], vec![]);
    let beyond = alloc::BASE + alloc::ARENA + 50;
    assert_eq!(unsafe { abi::truncate(ram, beyond + 10) }, 0);
    let mut pos = 0;
    alloc::place(ram, &mut pos, beyond).unwrap();
    assert_eq!(unsafe { abi::write1(ram, 0x5A) }, 0);
    // P0 calls ensure immediately after open_ram, with the cursor at zero.
    pos = beyond + 1;
    alloc::place(ram, &mut pos, 0).unwrap();
    assert_eq!(alloc::ensure(ram), Ok(()));
    pos = 0;
    alloc::place(ram, &mut pos, beyond).unwrap();
    assert_eq!(unsafe { abi::read1(ram) }, 0x5A);
}
