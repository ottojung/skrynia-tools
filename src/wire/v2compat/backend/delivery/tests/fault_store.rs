use super::{Durable, Pending};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Default)]
pub struct Faulty(Rc<RefCell<State>>);
#[derive(Default)]
struct State {
    items: Vec<Pending>,
    mode: u8,
}
impl Faulty {
    pub fn set(&self, mode: u8) {
        self.0.borrow_mut().mode = mode;
    }
    pub fn stored(&self) -> Vec<Pending> {
        self.0.borrow().items.clone()
    }
}
impl Durable for Faulty {
    type Error = &'static str;
    fn load(&self) -> Result<Vec<Pending>, Self::Error> {
        if self.0.borrow().mode == 3 {
            return Err("load_failed");
        }
        Ok(self.stored())
    }
    fn save(&self, items: &[Pending]) -> Result<(), Self::Error> {
        let mut s = self.0.borrow_mut();
        match s.mode {
            1 => Err("not_saved"),
            2 => {
                s.items = items.to_vec();
                Err("ack_lost")
            }
            _ => {
                s.items = items.to_vec();
                Ok(())
            }
        }
    }
}
