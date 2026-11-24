//! Every COW mutation boundary preserves committed state.
use super::crash_support::CrashDisk;
use skrynia_native_service::index::{self, Scope};
pub fn key(n: u32) -> [u8; 32] {
    let mut b = [0; 32];
    b[28..].copy_from_slice(&n.to_be_bytes());
    b
}

#[test]
fn every_cow_mutation_boundary_preserves_committed_keys_and_bounded_space() {
    let s = Scope { base: "reg" };
    for after in [false, true] {
        for at in 1..=16 {
            let d = CrashDisk::default();
            index::insert(&d, s, key(1)).unwrap();
            index::insert(&d, s, key(2)).unwrap();
            for round in 0..20 {
                // Failure may occur before or after a durable mutation, including
                // a committed meta update whose caller never received its ACK.
                d.arm(at, after);
                let _ = index::insert(&d, s, key(3));
                d.recover_io();
                index::insert(&d, s, key(3)).unwrap();
                assert_eq!(
                    index::page(&d, s, &None, 256).unwrap(),
                    vec![key(1), key(2), key(3)],
                    "at={at} after={after} round={round}"
                );
                d.arm(at, after);
                let _ = index::remove(&d, s, key(3));
                d.recover_io();
                index::remove(&d, s, key(3)).unwrap();
                assert_eq!(
                    index::page(&d, s, &None, 256).unwrap(),
                    vec![key(1), key(2)],
                    "at={at} after={after} round={round}"
                );
            }
            assert!(
                d.files.borrow().len() <= 16,
                "{} files after repeated retries; at={at} after={after}",
                d.files.borrow().len()
            );
        }
    }
}
