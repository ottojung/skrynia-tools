use super::{BlobRef, Snapshot, SnapshotBackend, SnapshotStore, digest};
use anyhow::Result;
use std::collections::BTreeMap;
impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    pub(super) fn prepare(
        &self,
        old: &Snapshot,
        txid: &str,
        desired: &BTreeMap<String, Vec<u8>>,
    ) -> Result<BTreeMap<String, BlobRef>> {
        let mut refs = BTreeMap::new();
        for (name, bytes) in desired {
            let item = if old.values.get(name) == Some(bytes) {
                old.refs
                    .get(name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("reference missing"))?
            } else {
                let key = self.blob_key(txid, name)?;
                self.stage(&key, bytes)?;
                BlobRef {
                    key,
                    sha256: digest(bytes),
                }
            };
            refs.insert(name.clone(), item);
        }
        Ok(refs)
    }
}

impl Snapshot {
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn values(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.values
    }
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.values.get(name).map(Vec::as_slice)
    }
}
