use super::*;
#[test]
fn fail_closed_on_uncertain_marker() {
    let dir = tmp("uncertain");
    let disk = FsDisk(dir.clone());
    let token = [3u8; 32];
    v2auth::bootstrap_root(&disk, &token).unwrap();
    std::fs::write(dir.join("v2auth/uncertain"), b"1").unwrap();
    let req = MutateReq {
        token,
        key: "a.txt",
        kind: MutateKind::Create,
        expected: None,
        body: b"x",
    };
    assert!(matches!(
        v2auth::mutate(&disk, req),
        Err(v2auth::MutateErr::Uncertain)
    ));
    let disk2 = FsDisk(dir.clone());
    assert!(matches!(
        v2auth::bootstrap_root(&disk2, &[9u8; 32]),
        Err(v2auth::StateErr::Uncertain)
    ));
    let _ = fs::remove_dir_all(dir);
}
