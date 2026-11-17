use super::{Keys, client, landing, stable};
use anyhow::Result;
pub fn ensure(client: &client::SkryniaClient, keys: &Keys) -> Result<()> {
    stable::put(client, &keys.entry(), landing().as_bytes())
}
