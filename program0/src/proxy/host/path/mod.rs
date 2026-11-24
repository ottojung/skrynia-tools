// Generic wire-path hygiene only. The kernel fs_delegate_scope remains the
// authority; P0 does not encode application grants or reserved-root policy.
pub(super) fn valid(path: &[u8]) -> bool {
    if path.is_empty() || path.len() > 4096 || path.contains(&0) || path.contains(&b'\\') {
        return false;
    }
    if path[0] == b'/' || path.ends_with(b"/") || path.windows(2).any(|w| w == b"//") {
        return false;
    }
    core::str::from_utf8(path).is_ok()
        && path
            .split(|&b| b == b'/')
            .all(|p| !p.is_empty() && p != b"." && p != b"..")
}
#[cfg(test)]
pub(super) use super::wire;
#[cfg(test)]
mod opcode_test {
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
}
