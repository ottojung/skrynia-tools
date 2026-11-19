use super::*;

#[test]
fn activation_requires_unchanged_pointer_revision() {
    let (r, d, l, a, id) = fixture();
    let hash = r.stage_release(&d, &l, &a, b"second").unwrap();
    r.seal_release(&d, &l, &a, &hash).unwrap();
    let changed = call(
        &r,
        &d,
        &l,
        Request::ActivateCode {
            admin_cap: a.clone(),
            expected_revision: 1,
            target: hash.to_vec(),
        },
    );
    assert_eq!(changed.status, STATUS_OK);
    assert_eq!(store::decode_active(&changed.body).unwrap().revision, 2);
    assert_eq!(r.lookup(&d, &id).unwrap(), b"second");
    let stale = call(
        &r,
        &d,
        &l,
        Request::ActivateCode {
            admin_cap: a,
            expected_revision: 1,
            target: hash.to_vec(),
        },
    );
    assert_eq!(stale.status, STATUS_ERR);
    assert_eq!(stale.body.first(), Some(&10));
}
