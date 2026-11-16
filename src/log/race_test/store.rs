use super::{AtomicUsize, Barrier, CreateResult, Fake, Ordering, Result, Store};

pub struct Coordinated {
    inner: Fake,
    barrier: Barrier,
    hits: AtomicUsize,
}
impl Default for Coordinated {
    fn default() -> Self {
        Self {
            inner: Fake::default(),
            barrier: Barrier::new(2),
            hits: AtomicUsize::new(0),
        }
    }
}
impl Store for Coordinated {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let result = self.inner.get(key)?;
        if key.ends_with(".000000000001")
            && result.is_none()
            && self.hits.fetch_add(1, Ordering::SeqCst) < 2
        {
            self.barrier.wait();
        }
        Ok(result)
    }
    fn create(&self, key: &str, bytes: &[u8]) -> Result<CreateResult> {
        self.inner.create(key, bytes)
    }
}
