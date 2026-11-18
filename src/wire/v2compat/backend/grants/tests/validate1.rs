use super::*;
#[test]
fn graph_accepts_safe_delegation() {
    assert!(Authority::from_grants([root(), child()]).is_ok());
}
#[test]
fn graph_rejects_orphans_cycles_and_duplicates() {
    let mut c = child();
    c.parent = Some("missing".into());
    assert_eq!(
        Authority::from_grants([root(), c]).err(),
        Some("orphan_grant")
    );
    let mut r = root();
    r.parent = Some("child".into());
    assert_eq!(
        Authority::from_grants([r, child()]).err(),
        Some("invalid_roots")
    );
    assert_eq!(
        Authority::from_grants([root(), root()]).err(),
        Some("duplicate_grant_id")
    );
}
