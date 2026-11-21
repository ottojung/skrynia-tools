use std::collections::BTreeMap;
use theorem_lab_feedback_p2::{Entry, MAX_FEEDBACK, State, apply};

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
fn idempotent_merge() {
    let e = entry("e1");
    let (s1, created) = apply(None, e.clone(), BTreeMap::new(), Some("t1".into())).unwrap();
    assert!(created);
    let (s2, created2) = apply(Some(&s1), e, BTreeMap::new(), Some("t2".into())).unwrap();
    assert!(!created2);
    let st: State = serde_json::from_slice(&s2).unwrap();
    assert_eq!(st.feedback.len(), 1);
    assert_eq!(st.updated_at.as_deref(), Some("t2"));
}

#[test]
fn quota_limits() {
    let mut st = State::empty();
    for i in 0..MAX_FEEDBACK {
        st.feedback.push(entry(&format!("id-{i}")));
    }
    let bytes = serde_json::to_vec(&st).unwrap();
    let err = apply(Some(&bytes), entry("new"), BTreeMap::new(), None).unwrap_err();
    assert_eq!(err, "quota_feedback");
}
