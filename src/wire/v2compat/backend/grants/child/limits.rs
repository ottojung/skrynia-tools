use super::CompatError;
use serde_json::Value;
pub fn parse(v: &Value) -> Result<(u64, u64), CompatError> {
    let bad = || CompatError::new(400, "invalid_grant_limits");
    let bytes = v.get("maxBytes").and_then(Value::as_u64).ok_or_else(bad)?;
    let objects = v
        .get("maxObjects")
        .and_then(Value::as_u64)
        .ok_or_else(bad)?;
    const MAX: u64 = 9007199254740991;
    if bytes < 2048 || objects == 0 || bytes > MAX || objects > MAX {
        return Err(bad());
    }
    Ok((bytes, objects))
}
