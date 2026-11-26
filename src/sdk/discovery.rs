//! Parse the discovery shapes used by GET /syscalls_list and POST /version.
use super::typed::{Signature, ValType};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    MissingKey,
    BadShape,
    BadType,
    DuplicateName,
}

fn types(value: &Value) -> Result<Vec<ValType>, DiscoveryError> {
    let array = value.as_array().ok_or(DiscoveryError::BadShape)?;
    array
        .iter()
        .map(|item| {
            let text = item.as_str().ok_or(DiscoveryError::BadType)?;
            ValType::parse(text).ok_or(DiscoveryError::BadType)
        })
        .collect()
}

fn signatures(root: &Value, key: &str) -> Result<Vec<Signature>, DiscoveryError> {
    let array = root
        .get(key)
        .ok_or(DiscoveryError::MissingKey)?
        .as_array()
        .ok_or(DiscoveryError::BadShape)?;
    let mut found = Vec::with_capacity(array.len());
    for item in array {
        let name = item
            .get("name")
            .and_then(Value::as_str)
            .ok_or(DiscoveryError::BadShape)?;
        if found.iter().any(|sig: &Signature| sig.name == name) {
            return Err(DiscoveryError::DuplicateName);
        }
        let params = types(item.get("params").ok_or(DiscoveryError::BadShape)?)?;
        let results = types(item.get("results").ok_or(DiscoveryError::BadShape)?)?;
        found.push(Signature::new(name, &params, &results));
    }
    Ok(found)
}

/// GET /syscalls_list: {"functions":[{"name", "params", "results"}]}.
pub fn parse_list(root: &Value) -> Result<Vec<Signature>, DiscoveryError> {
    signatures(root, "functions")
}

/// POST /version: the same descriptors under "syscalls", plus build metadata.
pub fn parse_version(root: &Value) -> Result<Vec<Signature>, DiscoveryError> {
    signatures(root, "syscalls")
}
