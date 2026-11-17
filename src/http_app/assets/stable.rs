use super::{Keys, client};
use anyhow::{Result, bail};
use client::{CreateResult, Store};
pub fn put(client: &client::SkryniaClient, key: &str, desired: &[u8]) -> Result<()> {
    match client.create(key, desired)? {
        CreateResult::Created => Ok(()),
        CreateResult::Exists => {
            let old = client
                .get(key)?
                .ok_or_else(|| anyhow::anyhow!("object vanished"))?;
            if old == desired {
                return Ok(());
            }
            let digest = client::SkryniaClient::digest(&old);
            if !client.compare_replace(key, desired, &digest)? {
                bail!("stable asset changed concurrently");
            }
            Ok(())
        }
    }
}
pub fn publish(
    client: &client::SkryniaClient,
    name: &str,
    asset: &str,
    bytes: &[u8],
) -> Result<String> {
    if !matches!(asset, "sw.js" | "manifest.webmanifest" | "icon.svg") {
        bail!("stable asset must be sw.js, manifest.webmanifest or icon.svg");
    }
    if bytes.is_empty() || bytes.len() > 5_000_000 {
        bail!("stable asset must be nonempty and at most 5 MB");
    }
    let key = format!("{}/{}", Keys::new(name)?.prefix, asset);
    put(client, &key, bytes)?;
    Ok(key)
}
