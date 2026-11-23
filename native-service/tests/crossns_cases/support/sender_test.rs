//! Genuine sender API proof for disjoint-source fixed-slot wake generations.
use super::{Mem, wake_setup};
use crate::crossns_cases::hlp::pw;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, read_cap};

#[test]
fn stale_sender_ack_preserves_cross_namespace_wake() {
    let (d, wa, wb, ga, gb) = wake_setup();
    let sender = [33u8; 32];
    let wake_ns = read_cap(&[1u8; 32]);
    let key = push::WAKE_KEY;
    push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Sender,
        &[1u8; 32],
        &sender,
        key,
        &[0u8; 32],
    )
    .unwrap();
    pw(&d, &wa, b"a", b"v", &ga, key).unwrap();
    let list = || push::hint_list(&d, &NoLock, "d", &sender, &wake_ns, key).unwrap();
    let first = list();
    assert_eq!(u32::from_le_bytes(first[..4].try_into().unwrap()), 1);
    let g1 = u64::from_le_bytes(first[4..12].try_into().unwrap());
    assert_eq!(
        push::hint_read(&d, &NoLock, "d", &sender, &wake_ns, key, g1).unwrap(),
        vec![0u8; 32]
    );
    pw(&d, &wb, b"b", b"v", &gb, key).unwrap();
    assert!(push::hint_read(&d, &NoLock, "d", &sender, &wake_ns, key, g1).is_err());
    push::hint_ack(&d, &NoLock, "d", &sender, &wake_ns, key, g1).unwrap();
    let second = list();
    assert_eq!(u32::from_le_bytes(second[..4].try_into().unwrap()), 1);
    let g2 = u64::from_le_bytes(second[4..12].try_into().unwrap());
    assert_eq!(g2, g1 + 1);
    push::hint_ack(&d, &NoLock, "d", &sender, &wake_ns, key, g2).unwrap();
    assert_eq!(u32::from_le_bytes(list()[..4].try_into().unwrap()), 0);
}
