use super::{Head, Manifest, Snapshot, SnapshotBackend, SnapshotStore, digest, keys};
use anyhow::{Result, bail};
use std::collections::BTreeMap;

impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    pub fn read(&self) -> Result<Snapshot> {
        let head_bytes = self
            .store
            .get(&self.head_key())?
            .ok_or_else(|| anyhow::anyhow!("snapshot root not initialized"))?;
        let head: Head = serde_json::from_slice(&head_bytes)?;
        if head.version != 1 || !keys::valid_sha(&head.manifest_sha256) {
            bail!("invalid snapshot head");
        }
        let manifest_bytes = self
            .store
            .get(&self.manifest_key(&head.txid)?)?
            .ok_or_else(|| anyhow::anyhow!("snapshot manifest missing"))?;
        if digest(&manifest_bytes) != head.manifest_sha256 {
            bail!("snapshot manifest hash mismatch");
        }
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
        if manifest.version != 1 {
            bail!("unsupported snapshot manifest version");
        }
        let mut values = BTreeMap::new();
        for (name, reference) in &manifest.items {
            self.verify_ref(name, &reference.key)?;
            let bytes = self
                .store
                .get(&reference.key)?
                .ok_or_else(|| anyhow::anyhow!("snapshot blob missing"))?;
            if digest(&bytes) != reference.sha256 {
                bail!("snapshot blob hash mismatch");
            }
            values.insert(name.clone(), bytes);
        }
        Ok(Snapshot {
            revision: head.revision,
            values,
            prefix: self.prefix.clone(),
            refs: manifest.items,
            head_sha256: digest(&head_bytes),
        })
    }
}
