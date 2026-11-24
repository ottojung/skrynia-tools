//! A sender that ACKs an observed generation cannot clear a newer one.
use crate::common::*;

#[test]
fn stale_ack_after_mutation_retains_new_wake() {
    let d = seeded();
    put(&d, b"first", b"v");
    let g1 = list(&d)[0];
    put(&d, b"second", b"v");
    let g2 = list(&d)[0];
    assert_eq!(list(&d).len(), 1, "one fixed slot, never a backlog");
    assert!(g2 > g1, "generation must advance under mutation");
    assert!(!read_ok(&d, g1), "superseded generation is not readable");
    ack(&d, &SENDER_A, g1);
    assert_eq!(list(&d), vec![g2], "stale ACK must retain newer wake");
    assert!(read_ok(&d, g2));
    ack(&d, &SENDER_A, g2);
    assert!(list(&d).is_empty(), "current ACK clears the slot");
}
