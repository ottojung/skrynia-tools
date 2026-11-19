use super::fakes::M;
use skrynia_tools::wire::v2compat::StateStore;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
#[derive(Clone, Default)]
pub struct Faulty {
    pub disk: M,
    mode: Arc<AtomicU8>,
}
impl Faulty {
    pub fn set(&self, value: u8) {
        self.mode.store(value, Ordering::SeqCst);
    }
}
impl StateStore for Faulty {
    type Error = String;
    fn load(&self) -> Result<Option<Vec<u8>>, String> {
        self.disk.load()
    }
    fn save(&self, bytes: &[u8]) -> Result<(), String> {
        match self.mode.load(Ordering::SeqCst) {
            1 => Err("not_saved".into()),
            2 => {
                self.disk.save(bytes)?;
                Err("reply_lost".into())
            }
            _ => self.disk.save(bytes),
        }
    }
}
