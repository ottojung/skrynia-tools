use skrynia_tools::wire::v2compat::{DurableBackend, RandomSource, StateStore};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
#[derive(Default, Clone)]
pub struct Mem(Arc<Mutex<Option<Vec<u8>>>>);
impl StateStore for Mem {
    type Error = String;
    fn load(&self) -> Result<Option<Vec<u8>>, String> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn save(&self, bytes: &[u8]) -> Result<(), String> {
        *self.0.lock().unwrap() = Some(bytes.to_vec());
        Ok(())
    }
}
pub struct R(AtomicU64);
impl RandomSource for R {
    type Error = String;
    fn fill(&self, out: &mut [u8]) -> Result<(), String> {
        for b in out {
            *b = self.0.fetch_add(1, Ordering::Relaxed) as u8;
        }
        Ok(())
    }
}
pub type Backend = DurableBackend<Mem, R>;
pub fn init(id: &str, issuer: &str) -> Result<Backend, String> {
    let b = Backend::open(Mem::default(), R(AtomicU64::new(2)))?;
    b.bootstrap_root(id, issuer).map_err(|e| e.code)?;
    Ok(b)
}
