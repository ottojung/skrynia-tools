use super::{client, release};
use client::Store;

use anyhow::{Context, Result, bail};
pub fn verify(client: &client::SkryniaClient, name: &str, id: &str) -> Result<()> {
    let data = release(client, name, id)?;
    for asset in data.files {
        let bytes = client.get(&asset.key)?.context("release asset missing")?;
        if bytes.len() as u64 != asset.bytes
            || client::SkryniaClient::digest(&bytes) != asset.sha256
        {
            bail!("release asset mismatch: {}", asset.path);
        }
    }
    Ok(())
}
