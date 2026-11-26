//! Framed descriptor ops 5-10 against deterministic fake capabilities.
use super::fake;
use super::support::{call, code, path_frame, setup, word};

#[test]
fn mkdir_delete_and_rename_are_relayed() {
    let (_, _, _) = setup(vec![(
        String::from("user/programs/blob/a.bin"),
        vec![1, 2, 3],
    )]);
    let mut mk = path_frame(5, "user/programs/appdata/x");
    mk.extend_from_slice(&word(4));
    assert_eq!(code(&call(&mk)), 0);
    fake::inspect(|s| assert!(s.dirs.iter().any(|d| d == "user/programs/appdata/x")));

    let mut rn = path_frame(7, "user/programs/blob/a.bin");
    rn.extend_from_slice(&word(24));
    rn.extend_from_slice(b"user/programs/blob/b.bin");
    assert_eq!(code(&call(&rn)), 0);
    fake::inspect(|s| {
        assert!(s.files.iter().any(|f| f.0 == "user/programs/blob/b.bin"));
        assert!(!s.files.iter().any(|f| f.0 == "user/programs/blob/a.bin"));
    });

    assert_eq!(code(&call(&path_frame(6, "user/programs/appdata/x"))), 0);
    fake::inspect(|s| assert!(!s.dirs.iter().any(|d| d == "user/programs/appdata/x")));
}

#[test]
fn open_directory_next_entry_and_sync_are_relayed() {
    let (_, _, _) = setup(vec![(
        String::from("user/programs/appdata/x/k.bin"),
        vec![9],
    )]);
    let mut mk = path_frame(5, "user/programs/appdata/x");
    mk.extend_from_slice(&word(4));
    assert_eq!(code(&call(&mk)), 0);

    let fd = code(&call(&path_frame(8, "user/programs/appdata/x")));
    assert!(fd >= 2);
    let mut nx = vec![9];
    nx.extend_from_slice(&word(fd as u32));
    nx.extend_from_slice(&word(256));
    let reply = call(&nx);
    assert_eq!(code(&reply), 5);
    assert_eq!(&reply[4..], b"k.bin");

    let mut sy = vec![10];
    sy.extend_from_slice(&word(fd as u32));
    assert_eq!(code(&call(&sy)), 0);
}

#[test]
fn truncated_frames_fail_closed() {
    let (_, _, _) = setup(vec![]);
    assert_eq!(code(&call(&[5, 1, 0, 0])), -1);
    assert_eq!(code(&call(&[9, 1, 0, 0])), -1);
    assert_eq!(code(&call(&[10])), -1);
}
