use super::Arguments;
use anyhow::{Result, bail};
use std::collections::BTreeMap;
pub fn parse() -> Result<Arguments> {
    let mut input = std::env::args().skip(1);
    let Some(group) = input.next() else {
        bail!("usage: skrynia-tools <app|object|log|admin> <action> [--key value]");
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
    Ok(Arguments {
        group,
        action,
        flags,
    })
}
