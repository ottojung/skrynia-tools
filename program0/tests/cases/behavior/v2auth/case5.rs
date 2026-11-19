use super::*;
use skrynia_program0::cap::Disk;
#[test]
fn malformed_stored_object_never_looks_absent() {
    let disk = crate::cases::support::MemDisk::default();
    let token = [15u8; 32];
    v2auth::bootstrap_root(&disk, &token).unwrap();
    let key = "kept";
    let path = format!("v2auth/objs/{}", hex::encode(sha256(key.as_bytes())));
    disk.write_atomic(&path, b"{corrupt").unwrap();
    let result = v2auth::mutate(
        &disk,
        MutateReq {
            token,
            key,
            kind: MutateKind::Create,
            expected: None,
            body: b"wrong",
        },
    );
    assert!(matches!(result, Err(v2auth::MutateErr::Decode)));
    assert_eq!(disk.read_all(&path).unwrap(), Some(b"{corrupt".to_vec()));
}
