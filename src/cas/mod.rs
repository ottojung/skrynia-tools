//! Single-key byte-hash compare-and-swap with explicit conflict handling.
use super::client::SkryniaClient;
use anyhow::{Result, bail};

pub struct CasCell<'a> {
    client: &'a SkryniaClient,
    key: String,
}
impl<'a> CasCell<'a> {
    pub fn open(client: &'a SkryniaClient, key: &str) -> Result<Self> {
        if key.is_empty()
            || key.len() > 255
            || !key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.')
            || key.starts_with('.')
        {
            bail!("invalid Skrynia object key");
        }
        Ok(Self {
            client,
            key: key.to_owned(),
        })
    }
    pub fn read(&self) -> Result<Option<(Vec<u8>, String)>> {
        Ok(self.client.read(&self.key)?.map(|b| {
            let digest = SkryniaClient::digest(&b);
            (b, digest)
        }))
    }
    pub fn compare_update(&self, digest: &str, bytes: &[u8]) -> Result<bool> {
        self.client.compare_replace(&self.key, bytes, digest)
    }
    /// The update callback must be pure: concurrent conflicts can replay it.
    pub fn update(
        &self,
        attempts: usize,
        mut next: impl FnMut(&[u8]) -> Result<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        for _ in 0..attempts {
            let (old, digest) = self
                .read()?
                .ok_or_else(|| anyhow::anyhow!("object is missing"))?;
            let proposed = next(&old)?;
            if self.compare_update(&digest, &proposed)? {
                return Ok(proposed);
            }
        }
        bail!("CAS update exhausted the permitted attempts")
    }
}
