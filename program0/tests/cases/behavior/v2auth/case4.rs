use super::*;
use skrynia_program0::{cap::Disk, v2auth::StateErr};
#[test]
fn interrupted_private_root_creation_recovers_without_rotation() {
    let disk = crate::cases::support::MemDisk::default();
    disk.write_atomic("v2auth/root-secret", &[17; 32]).unwrap();
    v2auth::ensure_root(&disk, &crate::cases::support::Rng(89)).unwrap();
    assert_eq!(
        disk.read_all("v2auth/root-secret").unwrap(),
        Some(vec![17; 32])
    );
    let sha = hex::encode(sha256(&[17; 32]));
    assert!(
        v2auth::state::load(&disk)
            .unwrap()
            .allows(&sha, "modify", "x")
    );
}
#[test]
fn corrupt_or_missing_private_root_secret_fails_closed() {
    let disk = crate::cases::support::MemDisk::default();
    disk.write_atomic("v2auth/root-secret", b"broken").unwrap();
    assert!(matches!(
        v2auth::ensure_root(&disk, &crate::cases::support::Rng(5)),
        Err(StateErr::Decode)
    ));
    let disk = crate::cases::support::MemDisk::default();
    v2auth::bootstrap_root(&disk, &[1; 32]).unwrap();
    assert!(matches!(
        v2auth::ensure_root(&disk, &crate::cases::support::Rng(5)),
        Err(StateErr::Uncertain)
    ));
}
