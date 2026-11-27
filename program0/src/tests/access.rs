use super::support::{call, code, path_frame, setup, word};
use super::{fake, invoke};

#[test]
fn qualified_operator_paths_never_reach_kernel() {
    let (_, _, _) = setup(vec![(String::from("operator/secret"), vec![42])]);
    let mut open = path_frame(1, "operator/secret");
    open.extend_from_slice(&word(1));
    assert_eq!(code(&call(&open)), -4);
    assert_eq!(code(&call(&path_frame(6, "operator/secret"))), -4);
    assert_eq!(code(&call(&path_frame(8, "operator/"))), -4);
    assert_eq!(code(&call(&path_frame(14, "operator/secret"))), -4);
    fake::inspect(|s| assert_eq!(s.files[0].1, vec![42]));
}

#[test]
fn only_explicitly_opened_descriptors_are_usable() {
    let (_, _, ram) = setup(vec![(String::from("user/data"), vec![42])]);
    let mut read = vec![2];
    read.extend_from_slice(&word(ram as u32));
    read.extend_from_slice(&word(1));
    assert_eq!(code(&call(&read)), -2);
    let mut open = path_frame(1, "user/data");
    open.extend_from_slice(&word(1));
    let fd = code(&call(&open));
    assert!(fd >= 2 && fd != ram);
    read[1..5].copy_from_slice(&word(fd as u32));
    assert_eq!(call(&read), vec![1, 0, 0, 0, 42]);
    let mut close = vec![4];
    close.extend_from_slice(&word(fd as u32));
    assert_eq!(code(&call(&close)), 0);
    assert_eq!(code(&call(&read)), -2);
    assert!(fake::inspect(|s| s.get(ram).is_some()));
    let next = code(&call(&open));
    assert!(next >= 2);
    invoke::leave();
    assert!(fake::inspect(|s| s.get(next).is_none()));
}

#[test]
fn truncated_frames_fail_closed() {
    let (_, _, _) = setup(vec![]);
    assert_eq!(code(&call(&[5, 1, 0, 0])), -1);
    assert_eq!(code(&call(&[9, 1, 0, 0])), -1);
    assert_eq!(code(&call(&[10])), -1);
}
