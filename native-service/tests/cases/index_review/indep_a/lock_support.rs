//! Lock-replay regression support.
use skrynia_native_service::cap::{Disk, Random};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

#[derive(Default)]
pub struct Mem(pub RefCell<BTreeMap<String, Vec<u8>>>);
impl Disk for Mem {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.borrow().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.borrow_mut().insert(p.into(), b.into());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.borrow_mut().remove(p);
        Ok(())
    }
    fn list(&self, path: &str) -> Result<Vec<String>, ()> {
        let prefix = format!("{}/", path.trim_end_matches('/'));
        let files = self.0.borrow();
        Ok(files
            .keys()
            .filter_map(|p| p.strip_prefix(&prefix))
            .filter_map(|p| p.split('/').next())
            .map(str::to_owned)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect())
    }
}
pub struct Stub;
impl Random for Stub {
    type Err = ();
    fn fill(&self, b: &mut [u8]) -> Result<(), ()> {
        b.fill(9);
        Ok(())
    }
}
