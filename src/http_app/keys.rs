use anyhow::{Result, bail};
#[derive(Clone)]
pub struct Keys {
    pub prefix: String,
}
impl Keys {
    pub fn new(name: &str) -> Result<Self> {
        if name.is_empty()
            || name.len() > 64
            || name.starts_with('-')
            || !name
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        {
            bail!("invalid app name");
        }
        Ok(Self {
            prefix: format!("apps/{name}"),
        })
    }
    pub fn pointer(&self) -> String {
        format!("{}/active.json", self.prefix)
    }
    pub fn entry(&self) -> String {
        format!("{}/index.html", self.prefix)
    }
    pub fn version(&self, id: &str) -> Result<String> {
        Self::validate(id)?;
        Ok(format!("{}/versions/{id}.json", self.prefix))
    }
    pub fn stamp(&self, id: &str) -> Result<String> {
        Self::validate(id)?;
        Ok(format!("{}/versions/{id}.ready", self.prefix))
    }
    pub fn validate(id: &str) -> Result<()> {
        if id.len() != 32
            || !id
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            bail!("invalid release id");
        }
        Ok(())
    }
}
