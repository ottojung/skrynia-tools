use super::{C, MemDur, Outcome, Policy, Queue, Tx};
#[test]
fn coalesces_ack_and_retries_with_persistence() {
    let dur = MemDur::default();
    let mut q = Queue::open(
        dur.clone(),
        Policy {
            max_failures: 3,
            base_ms: 10,
            max_ms: 100,
        },
    );
    q.enqueue("s1".into(), "https://e/1".into());
    q.enqueue("s1".into(), "https://e/1".into());
    assert_eq!(q.items.len(), 1);
    let mut tx = Tx(vec![Outcome::Retry, Outcome::Retry, Outcome::Ack]);
    q.run(&C(0), &mut tx);
    assert_eq!(q.items[0].attempts, 1);
    let mut q2 = Queue::open(
        dur.clone(),
        Policy {
            max_failures: 3,
            base_ms: 10,
            max_ms: 100,
        },
    );
    q2.run(&C(5), &mut tx);
    assert_eq!(q2.items.len(), 1);
    q2.run(&C(20), &mut tx);
    q2.run(&C(100), &mut tx);
    assert!(q2.items.is_empty());
}
