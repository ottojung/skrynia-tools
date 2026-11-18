use super::{Authority, CompatError, Spec, fields, limits, scope};
use serde_json::Value;
use std::collections::HashSet;
const OPS: [&str; 6] = [
    "create",
    "modify",
    "delete",
    "list",
    "delegate",
    "subscribe",
];
impl Authority {
    pub fn child_spec(&self, token: &str, value: &Value) -> Result<Spec, CompatError> {
        let parent = self.user(token)?;
        if !parent.operations.iter().any(|s| s == "delegate") {
            return Err(CompatError::new(403, "grant_operation_forbidden"));
        }
        let locations = fields::locations(value)?;
        if !scope::valid_scope(locations) {
            return Err(CompatError::new(400, "invalid_locations"));
        }
        if !scope::subset(locations, &parent.locations) {
            return Err(CompatError::new(403, "locations_outside_parent"));
        }
        let operations = fields::operations(value)?;
        let mut seen = HashSet::new();
        if operations
            .iter()
            .any(|op| !OPS.contains(&op.as_str()) || !seen.insert(op))
        {
            return Err(CompatError::new(400, "invalid_operations"));
        }
        if operations.iter().any(|op| !parent.operations.contains(op)) {
            return Err(CompatError::new(403, "operations_outside_parent"));
        }
        let (max_bytes, max_objects) = limits::parse(value)?;
        Ok(Spec {
            locations: locations.into(),
            operations,
            max_bytes,
            max_objects,
        })
    }
}
