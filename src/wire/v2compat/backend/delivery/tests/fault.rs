use super::fault_store::Faulty;
use super::{C, Outcome, Policy, Queue, Tx};
#[test]
fn failed_enqueue_latches_queue_and_preserves_memory() {
    let dur = Faulty::default();
    let mut q = Queue::open(dur.clone(), Policy::default()).unwrap();
    dur.set(1);
    assert_eq!(
        q.enqueue("s".into(), "https://e".into()),
        Err("state_uncertain")
    );
    assert!(matches!(q.pending(), Err("state_uncertain")));
    assert!(dur.stored().is_empty());
    assert!(matches!(q.due(0), Err("state_uncertain")));
    dur.set(0);
    assert_eq!(
        q.enqueue("t".into(), "https://e".into()),
        Err("state_uncertain")
    );
    let mut recovered = Queue::open(dur, Policy::default()).unwrap();
    recovered.enqueue("t".into(), "https://e".into()).unwrap();
}
#[test]
fn lost_delivery_ack_prevents_further_sends_until_restart() {
    let dur = Faulty::default();
    let mut q = Queue::open(dur.clone(), Policy::default()).unwrap();
    q.enqueue("s".into(), "https://e".into()).unwrap();
    dur.set(2);
    let mut tx = Tx(vec![Outcome::Ack]);
    assert_eq!(q.run(&C(0), &mut tx), Err("state_uncertain"));
    assert!(matches!(q.pending(), Err("state_uncertain")));
    assert!(dur.stored().is_empty());
    assert_eq!(q.run(&C(0), &mut tx), Err("state_uncertain"));
    dur.set(0);
    assert!(
        Queue::open(dur, Policy::default())
            .unwrap()
            .pending()
            .unwrap()
            .is_empty()
    );
}
