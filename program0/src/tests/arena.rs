//! Regression tests for staging bounds and monotone RAM growth.
use super::{abi, alloc, code, fake};

#[test]
fn stage_caps_at_the_slot_boundary() {
    let huge = vec![0xAA; (alloc::SLOT_SIZE - alloc::SLOT_CODE_AT + 1) as usize];
    let (_, _, ram) = fake::seed(
        vec![],
        vec![(String::from_utf8_lossy(code::GUEST_PATH).into_owned(), huge)],
    );
    alloc::ensure(ram).expect("arena");
    let (_slot, base) = alloc::acquire(ram).expect("slot");
    let mut pos = 0;
    code::put_path(ram, &mut pos, base + alloc::SLOT_PATH_AT, code::GUEST_PATH).unwrap();
    assert_eq!(
        code::stage(
            ram,
            &mut pos,
            base + alloc::SLOT_CODE_AT,
            base + alloc::SLOT_PATH_AT
        ),
        Err(alloc::ERR_RANGE)
    );
    assert!(pos <= base + alloc::SLOT_SIZE);
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
