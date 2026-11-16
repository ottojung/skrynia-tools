use super::{CreateResult, SnapshotBackend, Store};
use anyhow::Result;
use std::{collections::BTreeMap, sync::Mutex};
#[derive(Default)]
pub struct Fake {
    pub items: Mutex<BTreeMap<String, Vec<u8>>>,
    pub uncertain: Mutex<bool>,
    pub uncertain_create: Mutex<bool>,
    pub fail_cas_before: Mutex<bool>,
}
impl Store for Fake {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.items.lock().unwrap().get(key).cloned())
    }
    fn create(&self, key: &str, bytes: &[u8]) -> Result<CreateResult> {
        let mut map = self.items.lock().unwrap();
        if map.contains_key(key) {
            return Ok(CreateResult::Exists);
        }
        map.insert(key.into(), bytes.into());
        if std::mem::take(&mut *self.uncertain_create.lock().unwrap()) {
            anyhow::bail!("lost creation acknowledgement");
        }
        Ok(CreateResult::Created)
    }
}
impl SnapshotBackend for Fake {
    fn compare_head(&self, key: &str, bytes: &[u8], digest: &str) -> Result<bool> {
        if std::mem::take(&mut *self.fail_cas_before.lock().unwrap()) {
            anyhow::bail!("head reply lost before mutation");
        }
        let mut items = self.items.lock().unwrap();
        let Some(current) = items.get(key) else {
            anyhow::bail!("head absent");
        };
        if super::super::digest(current) != digest {
            return Ok(false);
        }
        items.insert(key.into(), bytes.into());
        if std::mem::take(&mut *self.uncertain.lock().unwrap()) {
            anyhow::bail!("lost CAS acknowledgment");
        }
        Ok(true)
    }
}
