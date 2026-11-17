use super::{Keys, client};
use anyhow::{Context, Result};
use std::collections::BTreeSet;
pub fn apps(client: &client::SkryniaClient) -> Result<Vec<String>> {
    let mut found = BTreeSet::new();
    let mut after = None;
    loop {
        let page = client.list_page("apps/*", after.as_deref())?;
        let keys = page["keys"].as_array().context("invalid app listing")?;
        for item in keys {
            let Some(key) = item.as_str() else {
                continue;
            };
            if let Some(name) = key
                .strip_prefix("apps/")
                .and_then(|s| s.strip_suffix("/active.json"))
                && Keys::new(name).is_ok()
                && !name.contains('/')
            {
                found.insert(name.to_owned());
            }
        }
        after = page["next"].as_str().map(str::to_owned);
        if after.is_none() {
            break;
        }
    }
    Ok(found.into_iter().collect())
}
