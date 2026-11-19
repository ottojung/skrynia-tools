use super::{Durable, Pending, Policy, Queue};
#[derive(Clone, Default)]
struct MemDur(std::rc::Rc<std::cell::RefCell<Vec<Pending>>>);
impl Durable for MemDur {
    type Error = String;
    fn load(&self) -> Result<Vec<Pending>, String> {
        Ok(self.0.borrow().clone())
    }
    fn save(&self, items: &[Pending]) -> Result<(), String> {
        *self.0.borrow_mut() = items.to_vec();
        Ok(())
    }
}
#[test]
fn remove_cancels_pending_for_subscription() {
    let dur = MemDur::default();
    let mut q = Queue::open(dur.clone(), Policy::default()).unwrap();
    q.enqueue("s1".into(), "https://e/1".into()).unwrap();
    q.enqueue("s2".into(), "https://e/2".into()).unwrap();
    assert_eq!(q.due(0).unwrap().len(), 2);
    q.remove("s1").unwrap();
    assert_eq!(q.pending().unwrap().len(), 1);
    assert_eq!(q.pending().unwrap()[0].sub_id, "s2");
    assert_eq!(dur.load().unwrap().len(), 1);
}
