//! Revoked grant and corrupt subscription record fail closed.
use super::support::setup;
use skrynia_native_service::wire::STATUS_ERR;

#[test]
fn op27_revoked_grant_and_corrupt_record_fail_closed() {
    let f = setup();
    let mut files = f.disk.0.lock().unwrap();
    let gp = files
        .keys()
        .find(|p| p.contains("pushgrant"))
        .cloned()
        .unwrap();
    files.remove(&gp);
    drop(files);
    let before = f.snapshot();
    assert_eq!(
        f.status(&f.key, &f.subscriber, &f.grant, f.expected()),
        (STATUS_ERR, vec![9])
    );
    assert_eq!(f.snapshot(), before);

    let g = setup();
    let before = g.snapshot();
    let mut files = g.disk.0.lock().unwrap();
    let sp = files
        .keys()
        .find(|p| p.contains("/subs/"))
        .cloned()
        .unwrap();
    files.insert(sp, b"garbage".to_vec());
    drop(files);
    let corrupted = g.snapshot();
    assert_eq!(
        g.status(&g.key, &g.subscriber, &g.grant, g.expected()),
        (STATUS_ERR, vec![9])
    );
    assert_eq!(g.snapshot(), corrupted);
    assert_ne!(corrupted, before);
}
