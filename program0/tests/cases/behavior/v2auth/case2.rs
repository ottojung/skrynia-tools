use super::*;
#[test]
fn grant_auth_rejects_wrong_token() {
    let dir = tmp("auth");
    let disk = FsDisk(dir.clone());
    v2auth::bootstrap_root(&disk, &[1u8; 32]).unwrap();
    let req = MutateReq {
        token: [2u8; 32],
        key: "a.txt",
        kind: MutateKind::Create,
        expected: None,
        body: b"x",
    };
    assert!(matches!(
        v2auth::mutate(&disk, req),
        Err(v2auth::MutateErr::InvalidToken)
    ));
    let _ = fs::remove_dir_all(dir);
}
