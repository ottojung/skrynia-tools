use super::deploy::{self, DeploySpec};
use anyhow::{Result, bail};
use std::io::Read;

pub fn deploy() -> Result<()> {
    let mut bytes = Vec::new();
    std::io::stdin().take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        bail!("deployment specification exceeds 16 KiB");
    }
    let spec: DeploySpec = serde_json::from_slice(&bytes)?;
    println!("{}", serde_json::to_string_pretty(&deploy::deploy(&spec)?)?);
    Ok(())
}
