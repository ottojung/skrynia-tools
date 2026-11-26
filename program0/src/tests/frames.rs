//! Strict frame validation: no effect from trailing, short or overlapping data.
use super::support::{call, code, path_frame, setup, word};
use super::{alloc, fake};

#[test]
fn random_fill_is_capped_to_one_slot() {
    let (_i, _o, _r) = setup(vec![]);
    let mut f = vec![11];
    f.extend_from_slice(&word(1));
    let bits = (alloc::SLOT_SIZE - alloc::SLOT_CODE_AT) * 8 + 8;
    f.extend_from_slice(&word(bits as u32));
    assert_eq!(code(&call(&f)), alloc::ERR_RANGE);
    let at = (alloc::BASE + alloc::SLOT_CODE_AT) as usize;
    fake::inspect(|s| assert!(s.ram[at..at + 8].iter().all(|&b| b == 0)));
}

#[test]
fn rename_second_path_cannot_reach_the_code_area() {
    let (_i, _o, _r) = setup(vec![]);
    let mut f = vec![7];
    f.extend_from_slice(&word(1000));
    f.extend_from_slice(&[b'a'; 1000]);
    f.extend_from_slice(&word(100));
    f.extend_from_slice(&[b'b'; 100]);
    assert_eq!(code(&call(&f)), alloc::ERR_RANGE);
    let at = (alloc::BASE + alloc::SLOT_CODE_AT) as usize;
    fake::inspect(|s| assert!(s.ram[at..at + 100].iter().all(|&b| b == 0)));
}

#[test]
fn trailing_or_incomplete_descriptor_frames_have_no_effect() {
    let (_i, _o, _r) = setup(vec![(String::from("programs/appdata/x"), vec![1])]);
    let mut mkdir = path_frame(5, "programs/appdata/y");
    mkdir.extend_from_slice(&word(4));
    mkdir.push(0);
    assert_eq!(code(&call(&mkdir)), -1);
    fake::inspect(|s| assert!(!s.dirs.iter().any(|d| d == "programs/appdata/y")));
    assert_eq!(code(&call(&path_frame(1, "programs/appdata/x"))), -1);
    let mut sync = vec![10];
    sync.extend_from_slice(&word(0));
    sync.push(9);
    assert_eq!(code(&call(&sync)), -1);
}
