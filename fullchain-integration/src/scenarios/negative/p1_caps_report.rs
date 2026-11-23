//! Assertions and evidence lines for the P1 capability negatives.
pub fn report(
    grant: u8,
    sibling: u8,
    sender: u8,
    fetch: u8,
    malformed: u8,
    version: u8,
) -> Vec<String> {
    assert_eq!((grant, sibling, sender, fetch, version), (9, 9, 9, 9, 1));
    assert!(matches!(malformed, 1 | 254 | 255));
    vec![
        "neg_wrong_grant=9".into(),
        "neg_sibling_key=9".into(),
        "neg_bad_sender=9".into(),
        "neg_bad_fetch=9".into(),
        format!("neg_malformed={malformed}"),
        format!("neg_wrong_version={version}"),
    ]
}
