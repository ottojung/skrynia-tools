use super::{Clock, Durable, Outcome, Pending, Policy, Queue, Transport};
#[derive(Clone, Default)]
struct MemDur(std::rc::Rc<std::cell::RefCell<Vec<Pending>>>);
impl Durable for MemDur {
    fn load(&self) -> Vec<Pending> {
        self.0.borrow().clone()
    }
    fn save(&self, items: &[Pending]) {
        *self.0.borrow_mut() = items.to_vec();
    }
}
struct C(u64);
impl Clock for C {
    fn now_ms(&self) -> u64 {
        self.0
    }
}
struct Tx(Vec<Outcome>);
impl Transport for Tx {
    fn send(&mut self, _: &str, _: &str) -> Outcome {
        self.0.remove(0)
    }
}
#[test]
fn remove_cancels_pending_for_subscription() {
    let dur = MemDur::default();
    let mut q = Queue::open(dur.clone(), Policy::default());
    q.enqueue("s1".into(), "https://e/1".into());
    q.enqueue("s2".into(), "https://e/2".into());
    assert_eq!(q.due(0).len(), 2);
    q.remove("s1");
    assert_eq!(q.items.len(), 1);
    assert_eq!(q.items[0].sub_id, "s2");
    assert_eq!(dur.load().len(), 1);
}
