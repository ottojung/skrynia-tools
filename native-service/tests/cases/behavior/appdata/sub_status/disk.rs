//! Self-contained disk and capability stubs for wire-level OP27 tests.
use skrynia_native_service::cap::{Disk, Eval, Random};
use std::{collections::BTreeMap, sync::Mutex};

#[derive(Default)]
pub struct Mem(pub Mutex<BTreeMap<String, Vec<u8>>>);
impl Disk for Mem {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.lock().unwrap().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.lock().unwrap().insert(p.into(), b.to_vec());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.lock().unwrap().remove(p);
        Ok(())
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        let p = format!("{}/", d.trim_end_matches('/'));
        Ok(self
            .0
            .lock()
            .unwrap()
            .keys()
            .filter_map(|k| k.strip_prefix(&p).map(str::to_string))
            .collect())
    }
}
pub struct Rng;
impl Random for Rng {
    type Err = ();
    fn fill(&self, b: &mut [u8]) -> Result<(), ()> {
        b.fill(7);
        Ok(())
    }
}
pub struct Eval0;
impl Eval for Eval0 {
    type Err = ();
    fn eval(
        &self,
        _: &[u8],
        _: &[u8],
        _: Option<&mut dyn FnMut(&[u8]) -> Result<Vec<u8>, i32>>,
    ) -> Result<Vec<u8>, ()> {
        Ok(Vec::new())
    }
}
