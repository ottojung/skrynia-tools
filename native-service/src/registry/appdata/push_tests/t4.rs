//! T4: sender identity is least-privilege — list/read/ack only inside one
//! explicitly scoped (namespace, key), with idempotent acknowledgement.
use super::support::seed_sender;
use super::{MemDisk, caps, push};

#[test]
fn sender_scope_is_exact_and_ack_is_idempotent() {
    let disk = MemDisk::default();
    let sender = [21u8; 32];
    let key = b"apps/demo/notice";
    let ns = crate::read_cap(&caps().0);
    seed_sender(&disk, &sender, key);
    let list = push::hint_list(&disk, &crate::NoLock, "d", &sender, &ns, key).unwrap();
    assert_eq!(list.len(), 12);
    let seq = u64::from_le_bytes(list[4..12].try_into().unwrap());
    assert_eq!(seq, 1);
    assert_eq!(
        push::hint_read(&disk, &crate::NoLock, "d", &sender, &ns, key, seq).unwrap(),
        vec![5u8; 32]
    );
    assert!(push::hint_list(&disk, &crate::NoLock, "d", &caps().1, &ns, key).is_err());
    assert!(push::hint_read(&disk, &crate::NoLock, "d", &sender, &[9u8; 32], key, seq).is_err());
    assert!(push::hint_read(&disk, &crate::NoLock, "d", &sender, &ns, b"other", seq).is_err());
    push::hint_ack(&disk, &crate::NoLock, "d", &sender, &ns, key, seq).unwrap();
    push::hint_ack(&disk, &crate::NoLock, "d", &sender, &ns, key, seq).unwrap();
    assert!(push::hint_read(&disk, &crate::NoLock, "d", &sender, &ns, key, seq).is_err());
}
