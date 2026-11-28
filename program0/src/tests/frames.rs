//! Strict frame validation: no effect from trailing, short or overlapping data.
use super::support::{call, code, path_frame, setup, word};
use super::{alloc, code as guest_code, fake};

#[test]
fn random_fill_keeps_existing_request_bound() {
    let (_i, _o, _r) = setup(vec![]);
    let mut f = vec![11];
    let bits = guest_code::MAX_CODE * 8 + 8;
    f.extend_from_slice(&word(bits as u32));
    assert_eq!(code(&call(&f)), alloc::ERR_RANGE);
    fake::inspect(|s| assert_eq!(s.ram.len(), alloc::META_END as usize));
}

#[test]
fn random_fill_returns_the_requested_bytes() {
    let (_i, _o, _r) = setup(vec![]);
    let mut f = vec![11];
    f.extend_from_slice(&word(128));
    let r = call(&f);
    assert_eq!(code(&r), 16);
    assert_eq!(&r[4..], &[0xa5; 16]);
    f.push(0);
    assert_eq!(code(&call(&f)), -1);
}

#[test]
fn rename_uses_independent_private_path_buffers() {
    let old = format!("user/{}old", "a/".repeat(299));
    let new = format!("user/{}new", "b/".repeat(299));
    let (_i, _o, _r) = setup(vec![(old.clone(), vec![7])]);
    let mut f = path_frame(7, &old);
    f.extend_from_slice(&word(new.len() as u32));
    f.extend_from_slice(new.as_bytes());
    assert_eq!(code(&call(&f)), 0);
    fake::inspect(|s| {
        assert_eq!(s.files, vec![(new, vec![7])]);
        assert_eq!(s.ram.len(), alloc::META_END as usize);
    });
}

#[test]
fn trailing_or_incomplete_descriptor_frames_have_no_effect() {
    let (_i, _o, _r) = setup(vec![(String::from("user/programs/appdata/x"), vec![1])]);
    let mut mkdir = path_frame(5, "user/programs/appdata/y");
    mkdir.extend_from_slice(&word(4));
    mkdir.push(0);
    assert_eq!(code(&call(&mkdir)), -1);
    fake::inspect(|s| assert!(!s.dirs.iter().any(|d| d == "user/programs/appdata/y")));
    assert_eq!(code(&call(&path_frame(1, "user/programs/appdata/x"))), -1);
    let mut sync = vec![10];
    sync.extend_from_slice(&word(0));
    sync.push(9);
    assert_eq!(code(&call(&sync)), -1);
}
