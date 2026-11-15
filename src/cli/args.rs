use anyhow::{Result, bail};
use std::collections::BTreeMap;

pub struct Arguments {
    pub group: String,
    pub action: String,
    flags: BTreeMap<String, String>,
}
impl Arguments {
    pub fn parse() -> Result<Self> {
        let mut input = std::env::args().skip(1);
        let Some(group) = input.next() else {
            bail!("usage: skrynia <app|object|log|admin> <action> [--key value]");
        };
        let Some(action) = input.next() else {
            bail!("missing action after {group}");
        };
        let mut flags = BTreeMap::new();
        while let Some(key) = input.next() {
            if !key.starts_with("--") || key.len() <= 2 {
                bail!("invalid option {key}");
            }
            let Some(value) = input.next() else {
                bail!("missing value for {key}");
            };
            if flags.insert(key, value).is_some() {
                bail!("duplicate option");
            }
        }
        Ok(Self {
            group,
            action,
            flags,
        })
    }
    pub fn required(&self, name: &str) -> Result<&str> {
        self.optional(name)
            .ok_or_else(|| anyhow::anyhow!("missing --{name}"))
    }
    pub fn optional(&self, name: &str) -> Option<&str> {
        self.flags.get(&format!("--{name}")).map(String::as_str)
    }
    pub fn locations(&self) -> Result<&str> {
        self.optional("locations")
            .or(self.optional("key"))
            .ok_or_else(|| anyhow::anyhow!("missing --locations"))
    }
    pub fn number(&self, name: &str) -> Result<u64> {
        Ok(self.required(name)?.parse()?)
    }
}
