use super::*;
use skrynia_program0::cap::Disk;
#[test]
fn mismatched_stored_object_key_cannot_be_replaced() {
    let disk = crate::cases::support::MemDisk::default();
    let token = [16u8; 32];
    v2auth::bootstrap_root(&disk, &token).unwrap();
    let path = format!("v2auth/objs/{}", hex::encode(sha256(b"real")));
    let rec = br#"{"key":"another","body":[1,2,3]}"#;
    disk.write_atomic(&path, rec).unwrap();
    let result = v2auth::mutate(
        &disk,
        MutateReq {
            token,
            key: "real",
            kind: MutateKind::Replace,
            expected: None,
            body: b"wrong",
        },
    );
    assert!(matches!(result, Err(v2auth::MutateErr::Decode)));
    assert_eq!(disk.read_all(&path).unwrap(), Some(rec.to_vec()));
}
