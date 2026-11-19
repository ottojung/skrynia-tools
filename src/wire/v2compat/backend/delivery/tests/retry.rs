use super::{C, MemDur, Outcome, Policy, Queue, Tx};
#[test]
fn removed_after_max_failures_and_gone() {
    let dur = MemDur::default();
    let mut q = Queue::open(
        dur.clone(),
        Policy {
            max_failures: 2,
            base_ms: 1,
            max_ms: 10,
        },
    );
    q.enqueue("s1".into(), "https://e".into());
    let mut tx = Tx(vec![Outcome::Retry, Outcome::Retry]);
    q.run(&C(0), &mut tx);
    q.run(&C(10), &mut tx);
    assert!(q.items.is_empty());
    q.enqueue("s2".into(), "https://e".into());
    let mut tx2 = Tx(vec![Outcome::Gone]);
    q.run(&C(20), &mut tx2);
    assert!(q.items.is_empty());
}
