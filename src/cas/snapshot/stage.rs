use super::{CreateResult, SnapshotBackend, SnapshotStore};
use anyhow::{Result, bail};

impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    /// Create-only immutable staging, with exact-byte retry reconciliation.
    pub(super) fn stage(&self, key: &str, bytes: &[u8]) -> Result<()> {
        match self.store.create(key, bytes) {
            Ok(CreateResult::Created) => Ok(()),
            Ok(CreateResult::Exists) => match self.store.get(key)? {
                Some(existing) if existing == bytes => Ok(()),
                Some(_) => bail!("immutable snapshot key collision"),
                None => bail!("snapshot staging key disappeared"),
            },
            Err(error) => match self.store.get(key) {
                Ok(Some(existing)) if existing == bytes => Ok(()),
                Ok(Some(_)) => bail!("snapshot key collision after uncertain staging"),
                Ok(None) => Err(error.context("snapshot staging outcome uncertain")),
                Err(check) => {
                    Err(error.context(format!("snapshot staging reconciliation failed: {check}")))
                }
            },
        }
    }
}
