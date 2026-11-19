use super::fault_store::Faulty;
use super::{Policy, Queue};
#[test]
fn failed_load_never_starts_delivery() {
    let dur = Faulty::default();
    dur.set(3);
    assert!(matches!(
        Queue::open(dur, Policy::default()),
        Err("load_failed")
    ));
}
