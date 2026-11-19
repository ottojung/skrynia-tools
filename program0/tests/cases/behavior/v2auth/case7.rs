use super::*;
use skrynia_program0::cap::Disk;
#[test]
fn v2_key_grammar_is_enforced_before_disk_mutation() {
    let disk = crate::cases::support::MemDisk::default();
    let token = [22u8; 32];
    v2auth::bootstrap_root(&disk, &token).unwrap();
    for key in ["", ".secret", "x/.y", "x//y", "x/../y", "a b", "λ"] {
        let req = MutateReq {
            token,
            key,
            kind: MutateKind::Create,
            expected: None,
            body: b"x",
        };
        assert!(
            matches!(v2auth::mutate(&disk, req), Err(v2auth::MutateErr::BadKey)),
            "{key:?}"
        );
    }
    let long = "a".repeat(256);
    assert!(matches!(
        v2auth::mutate(
            &disk,
            MutateReq {
                token,
                key: &long,
                kind: MutateKind::Create,
                expected: None,
                body: b"x"
            }
        ),
        Err(v2auth::MutateErr::BadKey)
    ));
    assert!(
        v2auth::mutate(
            &disk,
            MutateReq {
                token,
                key: "a/b-c_1.txt",
                kind: MutateKind::Create,
                expected: None,
                body: b"x",
            }
        )
        .is_ok()
    );
    assert!(disk.read_all("v2auth/uncertain").unwrap().is_none());
}
