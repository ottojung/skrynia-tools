use theorem_lab_feedback_p2::{ExecError, STATUS_ERR, submit_json};

#[test]
fn structured_error_on_rejection() {
    let mut v = serde_json::json!({
        "v":1,"op":1,"invokeId":"11".repeat(32),
        "entry":{"id":"bad id","comment":"ok","htmlChain":[]}
    });
    v["entry"]["id"] = serde_json::json!("bad id");
    let r = submit_json(v.to_string().as_bytes(), |_, out| {
        let body = vec![1, STATUS_ERR, 1, 0, 0, 0, 8]; // GET absent
        out[..body.len()].copy_from_slice(&body);
        body.len() as i32
    });
    assert_eq!(r.unwrap_err(), ExecError::Rejected("bad_id"));
}
