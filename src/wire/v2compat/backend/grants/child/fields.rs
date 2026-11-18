use super::CompatError;
use serde_json::Value;
fn bad(code: &str) -> CompatError {
    CompatError::new(400, code)
}
pub fn locations(v: &Value) -> Result<&str, CompatError> {
    let obj = v.as_object().ok_or_else(|| bad("invalid_grant_spec"))?;
    if obj
        .keys()
        .any(|k| !["locations", "operations", "maxBytes", "maxObjects"].contains(&k.as_str()))
    {
        return Err(bad("invalid_grant_spec"));
    }
    obj.get("locations")
        .and_then(Value::as_str)
        .ok_or_else(|| bad("invalid_locations"))
}
pub fn operations(v: &Value) -> Result<Vec<String>, CompatError> {
    serde_json::from_value(
        v.get("operations")
            .cloned()
            .ok_or_else(|| bad("invalid_operations"))?,
    )
    .map_err(|_| bad("invalid_operations"))
}
