use super::{Commit, Head, Manifest, Snapshot, SnapshotBackend, SnapshotStore, digest, keys};
use anyhow::{Result, bail};
use std::collections::BTreeMap;

impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    pub fn commit(
        &self,
        old: &Snapshot,
        txid: &str,
        desired: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Commit> {
        keys::valid_txid(txid)?;
        if old.prefix != self.prefix {
            bail!("snapshot belongs to a different store");
        }
        if desired.len() > 10000 {
            bail!("snapshot exceeds 10000 members");
        }
        let refs = self.prepare(old, txid, desired)?;
        let manifest = serde_json::to_vec(&Manifest {
            version: 1,
            items: refs,
        })?;
        self.stage(&self.manifest_key(txid)?, &manifest)?;
        let head = Head {
            version: 1,
            revision: old
                .revision
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("snapshot revision overflow"))?,
            txid: txid.into(),
            manifest_sha256: digest(&manifest),
        };
        let bytes = serde_json::to_vec(&head)?;
        match self
            .store
            .compare_head(&self.head_key(), &bytes, &old.head_sha256)
        {
            Ok(true) => Ok(Commit::Committed),
            Ok(false) => match self.store.get(&self.head_key()) {
                Ok(Some(current)) if current == bytes => Ok(Commit::Committed),
                Ok(_) => Ok(Commit::Conflict),
                Err(error) => Err(error.context("cannot reconcile snapshot head conflict")),
            },
            Err(error) => match self.store.get(&self.head_key()) {
                Ok(Some(current)) if current == bytes => Ok(Commit::Committed),
                _ => Err(error.context("snapshot head CAS outcome unknown; inspect before retry")),
            },
        }
    }
}
