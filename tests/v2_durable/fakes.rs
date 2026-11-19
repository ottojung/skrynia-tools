use skrynia_tools::wire::v2compat::{RandomSource, StateStore};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

#[derive(Clone, Default)]
pub struct M(pub Arc<Mutex<Option<Vec<u8>>>>);

impl StateStore for M {
    type Error = String;
    fn load(&self) -> Result<Option<Vec<u8>>, String> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn save(&self, b: &[u8]) -> Result<(), String> {
        *self.0.lock().unwrap() = Some(b.into());
        Ok(())
    }
}

pub struct R(pub AtomicU64);

impl RandomSource for R {
    type Error = String;
    fn fill(&self, o: &mut [u8]) -> Result<(), String> {
        for x in o {
            *x = self.0.fetch_add(1, Ordering::Relaxed) as u8;
        }
        Ok(())
    }
}
