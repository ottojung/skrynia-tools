use theorem_lab_feedback_p2::{ExecError, STATUS_ERR, STATUS_OK, submit_json};

fn input(id: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "v":1,"op":1,"invokeId":"11".repeat(32),
        "entry":{"id":id,"comment":"ok","pageUrl":"https://x","htmlChain":[]},
        "fragments":{}, "updatedAt":"2026-11-21T00:00:00Z"
    }))
    .unwrap()
}
fn resp(code: u8) -> Vec<u8> {
    vec![1, STATUS_ERR, 1, 0, 0, 0, code]
}

#[test]
fn cas_conflict_retry_then_ok() {
    let mut calls = 0usize;
    let r = submit_json(&input("e1"), |_, out| {
        calls += 1;
        let body = match calls {
            1 | 3 => resp(8),
            2 => resp(10),
            _ => {
                let mut r = vec![1, STATUS_OK, 32, 0, 0, 0];
                r.extend_from_slice(&[0u8; 32]);
                r
            }
        };
        out[..body.len()].copy_from_slice(&body);
        body.len() as i32
    })
    .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&r).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["retries"], 1);
    assert!(calls >= 4);
}

#[test]
fn cas_busy_after_retries() {
    let mut calls = 0usize;
    let r = submit_json(&input("e1"), |_, out| {
        calls += 1;
        let body = if calls % 2 == 1 { resp(8) } else { resp(10) };
        out[..body.len()].copy_from_slice(&body);
        body.len() as i32
    });
    assert_eq!(r.unwrap_err(), ExecError::Busy);
}
