use super::{DiscoveryError, MODULE, Signature, list};
use serde_json::Value;

pub fn version_abi(root: &Value) -> Result<&str, DiscoveryError> {
    let abi = root
        .get("abi")
        .and_then(Value::as_str)
        .ok_or(DiscoveryError::MissingAbi)?;
    if abi != MODULE {
        return Err(DiscoveryError::BadAbi);
    }
    Ok(abi)
}

pub fn parse_version(root: &Value) -> Result<Vec<Signature>, DiscoveryError> {
    version_abi(root)?;
    list::signatures(root, "syscalls")
}
