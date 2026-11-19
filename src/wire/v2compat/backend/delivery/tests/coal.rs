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
    )
    .unwrap();
    q.enqueue("s1".into(), "https://e/1".into()).unwrap();
    q.enqueue("s1".into(), "https://e/1".into()).unwrap();
    assert_eq!(q.pending().unwrap().len(), 1);
    let mut tx = Tx(vec![Outcome::Retry, Outcome::Retry, Outcome::Ack]);
    q.run(&C(0), &mut tx).unwrap();
    assert_eq!(q.pending().unwrap()[0].attempts, 1);
    let mut q2 = Queue::open(
        dur.clone(),
        Policy {
            max_failures: 3,
            base_ms: 10,
            max_ms: 100,
        },
    )
    .unwrap();
    q2.run(&C(5), &mut tx).unwrap();
    assert_eq!(q2.pending().unwrap().len(), 1);
    q2.run(&C(20), &mut tx).unwrap();
    q2.run(&C(100), &mut tx).unwrap();
    assert!(q2.pending().unwrap().is_empty());
}
