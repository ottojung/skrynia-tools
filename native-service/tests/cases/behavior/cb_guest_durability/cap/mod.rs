pub mod call;
pub mod disk_atom;
pub mod disk_read;
pub mod disk_rm;
pub mod disk_write;
mod dispatch;
mod replies;
pub mod wire;

#[test]
fn wire_codes_are_reachable() {
    let codes = [
        wire::READ,
        wire::NEXT_ENTRY,
        wire::TRY_LOCK,
        wire::RELEASE_LOCK,
    ];
    assert!(codes.iter().all(|&c| c != 0));
    const { assert!(wire::E_NOENT < 0) };
    assert!(wire::is_missing(wire::E_NOENT));
}

#[test]
fn sibling_helpers_are_reachable() {
    let req = vec![wire::MKDIR];
    assert_eq!(call::call(&req), call::call_with_capacity(&req, 8));
    assert_eq!(disk_read::read_all("user/missing"), Ok(None));
    assert_eq!(disk_rm::remove("user/missing"), Ok(()));
}
