use super::{Keys, Pointer, Release, client};
use client::Store;

use anyhow::{Context, Result, bail};
pub fn release(client: &client::SkryniaClient, name: &str, id: &str) -> Result<Release> {
    let key = Keys::new(name)?.version(id)?;
    let bytes = client.get(&key)?.context("release not found")?;
    let data: Release = serde_json::from_slice(&bytes)?;
    if data.schema != 1 || data.name != name || data.id != id {
        bail!("invalid release manifest");
    }
    Ok(data)
}
pub fn current(client: &client::SkryniaClient, name: &str) -> Result<Option<Release>> {
    let Some(bytes) = client.get(&Keys::new(name)?.pointer())? else {
        return Ok(None);
    };
    let ptr: Pointer = serde_json::from_slice(&bytes)?;
    if ptr.schema != 1 {
        bail!("unsupported active pointer");
    }
    Ok(Some(release(client, name, &ptr.release)?))
}
