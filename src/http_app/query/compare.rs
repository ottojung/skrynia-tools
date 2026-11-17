use super::{client, release};
use anyhow::Result;
use std::collections::BTreeMap;
pub fn compare(
    client: &client::SkryniaClient,
    name: &str,
    from: &str,
    to: &str,
) -> Result<serde_json::Value> {
    let first = release(client, name, from)?;
    let second = release(client, name, to)?;
    let old: BTreeMap<_, _> = first.files.iter().map(|a| (&a.path, &a.sha256)).collect();
    let new: BTreeMap<_, _> = second.files.iter().map(|a| (&a.path, &a.sha256)).collect();
    let added: Vec<_> = new
        .keys()
        .filter(|k| !old.contains_key(*k))
        .copied()
        .collect();
    let removed: Vec<_> = old
        .keys()
        .filter(|k| !new.contains_key(*k))
        .copied()
        .collect();
    let changed: Vec<_> = new
        .iter()
        .filter(|(k, v)| old.get(*k).is_some_and(|old| old != *v))
        .map(|(k, _)| *k)
        .collect();
    Ok(serde_json::json!({"from":from,"to":to,"added":added,"removed":removed,"changed":changed}))
}
