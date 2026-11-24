use super::{dispatch, open, paths, wire::*};
const VERIFIER: &[u8] = b"/security/program0.sha256";
fn code(out: Vec<u8>) -> i32 {
    i32::from_le_bytes(out[..4].try_into().unwrap())
}
fn req(op: u8, path: &[u8], tail: &[u8]) -> Vec<u8> {
    let mut r = vec![op];
    r.extend_from_slice(&(path.len() as u32).to_le_bytes());
    r.extend_from_slice(path);
    r.extend_from_slice(tail);
    r
}
fn open_req(p: &[u8]) -> Vec<u8> {
    req(OPEN_DISK, p, &1u32.to_le_bytes())
}
fn del_req(p: &[u8]) -> Vec<u8> {
    req(DELETE, p, &[])
}
fn rename_req(old: &[u8], newp: &[u8]) -> Vec<u8> {
    let mut r = req(RENAME, old, &[]);
    r.extend_from_slice(&(newp.len() as u32).to_le_bytes());
    r.extend_from_slice(newp);
    r
}
const BAD: &[&[u8]] = &[
    b"",
    b"/security/program0.sha256",
    b"a/../b",
    b"a\\b",
    b"a//b",
    b"a/",
    b"a\0b",
    b"\xff",
];
#[test]
fn unsafe_open_delete_rename_paths_denied_before_abi() {
    for p in BAD {
        assert_eq!(code(open::call(&open_req(p))), DENIED);
        assert_eq!(code(paths::delete(&del_req(p))), DENIED);
        assert_eq!(code(paths::rename(&rename_req(p, b"data/ok"))), DENIED);
        assert_eq!(code(paths::rename(&rename_req(b"data/ok", p))), DENIED);
    }
    assert_ne!(code(open::call(&open_req(b"data/ok"))), DENIED);
}
#[test]
fn leak_safe_nested_callback_isolation() {
    // Supplemental source guard for the LEAK->SAFE boundary: the generic
    // callback must not re-enter EVAL or open an out-of-scope/private path.
    // The authoritative compiled regression is the upstream kernel test
    // callback::nested::nested_guest_cannot_steal_root_disk_fd_after_callback
    // run by the compiled-stack CI job; this test only guards the local logic.
    assert_eq!(code(dispatch(&[EVAL], 4)), DENIED);
    assert_eq!(code(open::call(&open_req(b"a/../b"))), DENIED);
    assert_eq!(code(open::call(&open_req(VERIFIER))), DENIED);
}
