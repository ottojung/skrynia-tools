use super::{Keys, client, landing};
use anyhow::{Result, bail};
use client::{CreateResult, Store};
pub fn ensure(client: &client::SkryniaClient, keys: &Keys) -> Result<()> {
    let key = keys.entry();
    let desired = landing().as_bytes();
    match client.create(&key, desired)? {
        CreateResult::Created => Ok(()),
        CreateResult::Exists => {
            let old = client
                .get(&key)?
                .ok_or_else(|| anyhow::anyhow!("entry vanished"))?;
            if old == desired {
                return Ok(());
            }
            let digest = client::SkryniaClient::digest(&old);
            if !client.compare_replace(&key, desired, &digest)? {
                bail!("application entry changed concurrently");
            }
            Ok(())
        }
    }
}
