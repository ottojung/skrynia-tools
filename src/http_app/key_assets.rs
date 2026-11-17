use super::Keys;
use anyhow::{Result, bail};
impl Keys {
    pub fn asset(&self, id: &str, path: &str) -> Result<String> {
        Self::validate(id)?;
        let key = format!("{}/releases/{id}/{path}", self.prefix);
        if !crate::wire::valid_key(&key) {
            bail!("invalid release artifact path: {path}");
        }
        Ok(key)
    }
}
