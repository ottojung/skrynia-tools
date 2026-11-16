use super::CreateResult;
use super::{Head, Manifest, SnapshotBackend, SnapshotStore, digest, keys};
use anyhow::Result;
use std::collections::BTreeMap;

impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    pub fn initialize(&self, txid: &str) -> Result<()> {
        keys::valid_txid(txid)?;
        if self.store.get(&self.head_key())?.is_some() {
            self.read()?;
            return Ok(());
        }
        let manifest = Manifest {
            version: 1,
            items: BTreeMap::new(),
        };
        let encoded = serde_json::to_vec(&manifest)?;
        self.stage(&self.manifest_key(txid)?, &encoded)?;
        let head = Head {
            version: 1,
            revision: 0,
            txid: txid.into(),
            manifest_sha256: digest(&encoded),
        };
        let bytes = serde_json::to_vec(&head)?;
        match self.store.create(&self.head_key(), &bytes) {
            Ok(CreateResult::Created) => Ok(()),
            Ok(CreateResult::Exists) | Err(_) => {
                self.read()?;
                Ok(())
            }
        }
    }
}
