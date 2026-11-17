use anyhow::{Result, anyhow};
use std::collections::BTreeMap;
mod parse;
pub struct Arguments {
    pub group: String,
    pub action: String,
    flags: BTreeMap<String, String>,
}
impl Arguments {
    pub fn parse() -> Result<Self> {
        parse::parse()
    }
    pub fn required(&self, name: &str) -> Result<&str> {
        self.optional(name)
            .ok_or_else(|| anyhow!("missing --{name}"))
    }
    pub fn optional(&self, name: &str) -> Option<&str> {
        self.flags.get(&format!("--{name}")).map(String::as_str)
    }
    pub fn locations(&self) -> Result<&str> {
        self.optional("locations")
            .or(self.optional("key"))
            .ok_or_else(|| anyhow!("missing --locations"))
    }
    pub fn number(&self, name: &str) -> Result<u64> {
        Ok(self.required(name)?.parse()?)
    }
}
