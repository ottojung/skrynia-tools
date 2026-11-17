mod activate;
mod assets;
mod key_assets;
mod keys;
mod manifest;
mod publish;
mod query;
mod scan;
use super::client;
pub use activate::activate;
use anyhow::{Result, bail};
use client::Store;
pub use keys::Keys;
pub use manifest::{Asset, Pointer, Release};
pub use publish::deploy;
pub use query::{apps, compare, current, release, verify, versions};
pub use scan::scan;
pub fn create_exact(client: &client::SkryniaClient, key: &str, bytes: &[u8]) -> Result<()> {
    match client.create(key, bytes)? {
        client::CreateResult::Created => Ok(()),
        client::CreateResult::Exists if client.get(key)?.as_deref() == Some(bytes) => Ok(()),
        client::CreateResult::Exists => bail!("existing immutable object differs: {key}"),
    }
}
