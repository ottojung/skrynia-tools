use std::collections::BTreeMap;
use theorem_lab_feedback_p2::{Entry, ExecError, MAX_ID, MAX_TEXT, submit_json, validate_entry};

fn iid_hex() -> String {
    "11".repeat(32)
}
fn entry(id: &str) -> Entry {
    Entry {
        id: id.into(),
        comment: "ok".into(),
        page_url: "https://x".into(),
        location: None,
        anchor: None,
        html_chain: vec![],
    }
}

#[test]
fn contract_parse_rejects_bad() {
    assert_eq!(
        submit_json(b"{}", |_, _| -1).unwrap_err(),
        ExecError::BadInput
    );
    let mut v = serde_json::json!({"v":2,"op":1,"invokeId":iid_hex(),
        "entry":{"id":"a","comment":"c","htmlChain":[]}});
    assert_eq!(
        submit_json(v.to_string().as_bytes(), |_, _| -1).unwrap_err(),
        ExecError::BadInput
    );
    v["v"] = serde_json::json!(1);
    v["invokeId"] = serde_json::json!("zz");
    assert_eq!(
        submit_json(v.to_string().as_bytes(), |_, _| -1).unwrap_err(),
        ExecError::BadInput
    );
}

#[test]
fn id_and_bounds_validation() {
    let mut e = entry("bad id");
    let empty = BTreeMap::new();
    assert_eq!(validate_entry(&e, &empty).unwrap_err(), "bad_id");
    e.id = "a".repeat(MAX_ID + 1);
    assert_eq!(validate_entry(&e, &empty).unwrap_err(), "bad_id");
    e.id = "ok".into();
    e.comment = "".into();
    assert_eq!(validate_entry(&e, &empty).unwrap_err(), "bad_comment");
    e.comment = "x".repeat(MAX_TEXT + 1);
    assert_eq!(validate_entry(&e, &empty).unwrap_err(), "bad_comment");
}
