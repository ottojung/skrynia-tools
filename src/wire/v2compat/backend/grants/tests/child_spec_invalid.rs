use super::*;
use serde_json::json;
#[test]
fn child_spec_rejects_invalid_fields_and_limits() {
    let auth = authority();
    for (patch, code) in [
        (json!({"locations":null}), "invalid_locations"),
        (
            json!({"locations":"team/a","badField":1}),
            "invalid_grant_spec",
        ),
        (
            json!({"operations":["create","create"]}),
            "invalid_operations",
        ),
        (json!({"operations":["unknown"]}), "invalid_operations"),
        (json!({"maxBytes":2047}), "invalid_grant_limits"),
        (json!({"maxObjects":0}), "invalid_grant_limits"),
    ] {
        let mut doc = valid();
        for (k, v) in patch.as_object().unwrap() {
            doc.as_object_mut().unwrap().insert(k.clone(), v.clone());
        }
        assert_eq!(auth.child_spec("token", &doc).err().unwrap().code, code);
    }
}
#[test]
fn child_spec_rejects_outside_parent() {
    let auth = authority();
    for (patch, code) in [
        (json!({"locations":"other/a"}), "locations_outside_parent"),
        (
            json!({"operations":["modify"]}),
            "operations_outside_parent",
        ),
    ] {
        let mut doc = valid();
        for (k, v) in patch.as_object().unwrap() {
            doc.as_object_mut().unwrap().insert(k.clone(), v.clone());
        }
        assert_eq!(auth.child_spec("token", &doc).err().unwrap().code, code);
    }
}
