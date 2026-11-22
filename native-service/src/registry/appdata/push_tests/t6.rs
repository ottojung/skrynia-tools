//! T6: G2 delivery path — a scoped sender can fetch stored subscription
//! material for its exact (namespace, key), and no broader cap can.
use super::support::seed_sender;
use super::{MemDisk, caps, push};
use crate::wire::{OP_PUSH_SUB_FETCH, Request};

#[test]
fn sender_scoped_sub_fetch_is_exact() {
    let disk = MemDisk::default();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&caps().0);
    seed_sender(&disk, &sender, key);
    let out = push::sub_fetch(&disk, "d", &sender, &ns, key).unwrap();
    assert_eq!(u32::from_le_bytes(out[0..4].try_into().unwrap()), 1);
    let len = u32::from_le_bytes(out[4..8].try_into().unwrap()) as usize;
    assert_eq!(&out[8..8 + len], b"endpoint");
    assert!(push::sub_fetch(&disk, "d", &caps().1, &ns, key).is_err());
    assert!(push::sub_fetch(&disk, "d", &sender, &[9u8; 32], key).is_err());
    assert!(push::sub_fetch(&disk, "d", &sender, &ns, b"other").is_err());
}

#[test]
fn sub_fetch_wire_roundtrip() {
    let req = Request::SubFetch(vec![1u8; 32], vec![2u8; 32], b"apps/demo/notice".to_vec());
    let bytes = crate::wenc::encode_req(&req);
    assert_eq!(bytes[1], OP_PUSH_SUB_FETCH);
    assert_eq!(crate::wdec::decode_req(&bytes).unwrap(), req);
}
