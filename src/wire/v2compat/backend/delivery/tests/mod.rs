mod coal;
mod fault;
mod fault_load;
mod fault_store;
mod retry;
use super::{Clock, Durable, Outcome, Pending, Policy, Queue, Transport};
#[derive(Clone, Default)]
pub(super) struct MemDur(std::rc::Rc<std::cell::RefCell<Vec<Pending>>>);
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
