//! Red-green tests T1-T6 for exact-key grants, scoped sender hints and the
//! scoped subscription-fetch delivery path.
mod support;
mod t1;
mod t2;
mod t3;
mod t4;
mod t5;
mod t6;
mod t7;

pub(super) use super::cap::Disk;
pub(super) use super::path;
pub(super) use super::push;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
pub(super) struct MemDisk(Mutex<BTreeMap<String, Vec<u8>>>);

impl Disk for MemDisk {
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

/// writer cap, narrow grant cap, subscriber id.
pub(super) fn caps() -> ([u8; 32], [u8; 32], [u8; 32]) {
    ([3u8; 32], [7u8; 32], [11u8; 32])
}
