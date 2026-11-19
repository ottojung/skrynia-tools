use super::*;
#[test]
fn cas_auth_and_restart_recovery() {
    let dir = tmp("cas");
    let disk = FsDisk(dir.clone());
    let token = [7u8; 32];
    v2auth::bootstrap_root(&disk, &token).unwrap();
    let req = MutateReq {
        token,
        key: "a.txt",
        kind: MutateKind::Create,
        expected: None,
        body: b"v1",
    };
    v2auth::mutate(&disk, req).unwrap();
    let disk2 = FsDisk(dir.clone());
    let bad = MutateReq {
        token,
        key: "a.txt",
        kind: MutateKind::Replace,
        expected: Some([0u8; 32]),
        body: b"v2",
    };
    assert!(matches!(
        v2auth::mutate(&disk2, bad),
        Err(v2auth::MutateErr::ContentMismatch)
    ));
    let ok = MutateReq {
        token,
        key: "a.txt",
        kind: MutateKind::Replace,
        expected: Some(sha256(b"v1")),
        body: b"v2",
    };
    v2auth::mutate(&disk2, ok).unwrap();
    let _ = fs::remove_dir_all(dir);
}
