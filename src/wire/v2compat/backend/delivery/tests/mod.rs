mod coal;
mod retry;
use super::{Clock, Durable, Outcome, Pending, Policy, Queue, Transport};
#[derive(Clone, Default)]
pub(super) struct MemDur(std::rc::Rc<std::cell::RefCell<Vec<Pending>>>);
impl Durable for MemDur {
    fn load(&self) -> Vec<Pending> {
        self.0.borrow().clone()
    }
    fn save(&self, items: &[Pending]) {
        *self.0.borrow_mut() = items.to_vec();
    }
}
pub(super) struct C(pub(super) u64);
impl Clock for C {
    fn now_ms(&self) -> u64 {
        self.0
    }
}
pub(super) struct Tx(pub(super) Vec<Outcome>);
impl Transport for Tx {
    fn send(&mut self, _: &str, _: &str) -> Outcome {
        self.0.remove(0)
    }
}
