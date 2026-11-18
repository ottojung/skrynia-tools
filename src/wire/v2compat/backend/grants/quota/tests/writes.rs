use super::fixture;
#[test]
fn acting_child_is_limited_but_root_is_unlimited() {
    let (auth, mut objects) = fixture();
    assert!(
        auth.check_write_quota("child", "team/a", &objects, 300, &[])
            .is_ok()
    );
    assert_eq!(
        auth.check_write_quota("child", "team/a", &objects, 400, &[])
            .unwrap_err()
            .status,
        507
    );
    objects.insert("team/a".into(), 300);
    assert_eq!(
        auth.check_write_quota("child", "team/b", &objects, 1, &[])
            .unwrap_err()
            .code,
        "grant_quota_exceeded"
    );
    assert!(
        auth.check_write_quota("child", "team/a", &objects, 10, &[])
            .is_ok()
    );
    assert!(
        auth.check_write_quota("root", "team/b", &objects, 5000, &[])
            .is_ok()
    );
}
#[test]
fn integer_overflow_denies_capacity() {
    let (auth, mut objects) = fixture();
    objects.insert("team/a".into(), u64::MAX);
    assert_eq!(
        auth.check_write_quota("child", "team/b", &objects, 1, &[])
            .unwrap_err()
            .status,
        507
    );
}
