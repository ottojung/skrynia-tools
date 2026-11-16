use super::{SkryniaClient, SnapshotBackend, SnapshotStore, wire};
use anyhow::{Result, bail};
impl SnapshotBackend for SkryniaClient {
    fn compare_head(&self, key: &str, bytes: &[u8], digest: &str) -> Result<bool> {
        self.compare_replace(key, bytes, digest)
    }
}

impl<'a, S: SnapshotBackend> SnapshotStore<'a, S> {
    pub fn open(store: &'a S, prefix: &str) -> Result<Self> {
        if !wire::valid_prefix(prefix) || prefix.len() > 200 {
            bail!("invalid snapshot prefix");
        }
        Ok(Self {
            store,
            prefix: prefix.into(),
        })
    }
}
