use super::*;
use skrynia_program0::{
    cap::Disk,
    cap::Random,
    protocol::wire::{Request, STATUS_ERR},
};
#[test]
fn generated_root_credential_is_not_caller_supplied() {
    let disk = crate::cases::support::MemDisk::default();
    let rng = crate::cases::support::Rng(31);
    v2auth::ensure_root(&disk, &rng).unwrap();
    let secret = disk.read_all("v2auth/root-secret").unwrap().unwrap();
    assert_eq!(secret, vec![31; 32]);
    let sha = hex::encode(sha256(&secret));
    assert!(
        v2auth::state::load(&disk)
            .unwrap()
            .allows(&sha, "create", "x")
    );
    let message = v2auth::handle_wire(&disk, &rng, Request::V2Bootstrap { token: [0; 32] });
    let parsed = skrynia_program0::wdec::decode_res(&message).unwrap();
    assert_eq!(parsed.status, STATUS_ERR);
    assert_eq!(parsed.body, b"invalid_state");
    assert_eq!(
        disk.read_all("v2auth/root-secret").unwrap().unwrap(),
        secret
    );
}
