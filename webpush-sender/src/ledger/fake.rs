//! Deterministic injected store with fail-closed fault knobs for adversarial
//! tests. No ambient filesystem access; shares state through `Rc` on clone.
use super::{Error, LedgerStore, Result};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct Faults {
    pub load: Option<Error>,
    pub save: Option<Error>,
}

#[derive(Clone)]
pub struct MemoryLedgerStore {
    bytes: Rc<RefCell<Option<Vec<u8>>>>,
    faults: Rc<RefCell<Faults>>,
}

impl MemoryLedgerStore {
    pub fn new() -> Self {
        Self {
            bytes: Rc::new(RefCell::new(None)),
            faults: Rc::new(RefCell::new(Faults::default())),
        }
    }

    pub fn faults(&self) -> Rc<RefCell<Faults>> {
        self.faults.clone()
    }

    pub fn set_bytes(&self, bytes: Option<Vec<u8>>) {
        *self.bytes.borrow_mut() = bytes;
    }
}

impl Default for MemoryLedgerStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerStore for MemoryLedgerStore {
    fn load(&self) -> Result<Option<Vec<u8>>> {
        if let Some(e) = self.faults.borrow().load.clone() {
            return Err(e);
        }
        Ok(self.bytes.borrow().clone())
    }

    fn save(&self, bytes: &[u8]) -> Result<()> {
        if let Some(e) = self.faults.borrow().save.clone() {
            return Err(e);
        }
        *self.bytes.borrow_mut() = Some(bytes.to_vec());
        Ok(())
    }
}
