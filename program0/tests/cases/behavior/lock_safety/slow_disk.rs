use skrynia_program0::cap::Disk;
use std::sync::{Arc, Barrier, atomic::AtomicBool, atomic::Ordering};
use std::time::Duration;

pub struct SlowDisk<D> {
    pub inner: D,
    pub gate: Arc<Barrier>,
    pub armed: Arc<AtomicBool>,
}

impl<D: Disk> Disk for SlowDisk<D> {
    type Err = D::Err;
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, Self::Err> {
        self.inner.read_all(p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), Self::Err> {
        let r = self.inner.write_atomic(p, b)?;
        if self.armed.swap(false, Ordering::SeqCst) {
            self.gate.wait();
            std::thread::sleep(Duration::from_millis(120));
            self.gate.wait();
        }
        Ok(r)
    }
    fn remove(&self, p: &str) -> Result<(), Self::Err> {
        self.inner.remove(p)
    }
    fn list(&self, d: &str) -> Result<Vec<String>, Self::Err> {
        self.inner.list(d)
    }
}
