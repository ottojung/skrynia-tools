use super::{SnapshotBackend, SnapshotStore, wire};
use anyhow::{Result, bail};
impl<S: SnapshotBackend> SnapshotStore<'_, S> {
    pub(super) fn head_key(&self) -> String {
        format!("{}.head", self.prefix)
    }
    pub(super) fn manifest_key(&self, txid: &str) -> Result<String> {
        valid_txid(txid)?;
        Ok(format!("{}.manifest.{txid}", self.prefix))
    }
    pub(super) fn blob_key(&self, txid: &str, name: &str) -> Result<String> {
        valid_txid(txid)?;
        if !wire::valid_key(name) {
            bail!("invalid snapshot member name");
        }
        let key = format!("{}.blob.{txid}.{name}", self.prefix);
        if !wire::valid_key(&key) {
            bail!("snapshot member key exceeds limit");
        }
        Ok(key)
    }
    pub(super) fn verify_ref(&self, name: &str, key: &str) -> Result<()> {
        let value = key
            .strip_prefix(&(self.prefix.clone() + ".blob."))
            .ok_or_else(|| anyhow::anyhow!("snapshot reference outside prefix"))?;
        let (txid, stored_name) = value
            .split_once('.')
            .ok_or_else(|| anyhow::anyhow!("invalid snapshot blob reference"))?;
        if stored_name != name || self.blob_key(txid, name)? != key {
            bail!("invalid snapshot blob reference");
        }
        Ok(())
    }
}
pub fn valid_txid(id: &str) -> Result<()> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
    {
        bail!("transaction ID must be 32 lowercase hexadecimal characters");
    }
    Ok(())
}

pub fn valid_sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
}
