use super::*;
use serde_json::{Value, json};
pub(super) fn valid() -> Value {
    json!({"locations":"team/a","operations":["create"],
        "maxBytes":2048,"maxObjects":1})
}
#[test]
fn child_spec_restricts_authority() {
    let auth = authority();
    let spec = auth.child_spec("token", &valid()).unwrap();
    assert_eq!(spec.locations, "team/a");
    assert_eq!(spec.operations, ["create"]);
    assert_eq!(spec.max_bytes, 2048);
    assert_eq!(spec.max_objects, 1);
}
