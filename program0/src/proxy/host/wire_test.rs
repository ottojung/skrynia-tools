use super::wire::*;
#[test]
fn pr49_opcodes_frozen() {
    assert_eq!(OPEN_DISK, 1);
    assert_eq!(READ, 2);
    assert_eq!(WRITE, 3);
    assert_eq!(CLOSE, 4);
    assert_eq!(MKDIR, 5);
    assert_eq!(DELETE, 6);
    assert_eq!(RENAME, 7);
    assert_eq!(OPEN_DIRECTORY, 8);
    assert_eq!(NEXT_ENTRY, 9);
    assert_eq!(SYNC, 10);
    assert_eq!(RANDOM_FILL, 11);
    assert_eq!(TRY_LOCK, 12);
    assert_eq!(RELEASE_LOCK, 13);
}
