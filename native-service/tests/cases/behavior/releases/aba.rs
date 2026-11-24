use super::support::fixture;
use skrynia_native_service::store;
use skrynia_native_service::wire::AppPrecond;

#[test]
fn ready_history_allows_but_nonce_prevents_aba_and_revoke_reuse() {
    let (r, d, lock, admin, id) = fixture();
    let first = r.active_info(&d, &id).unwrap();
    assert_eq!(first.revision, 1);
    let h2 = r.stage_release(&d, &lock, &admin, b"second").unwrap();
    assert!(r.activate_release(&d, &lock, &admin, 1, &h2).is_err());
    assert_eq!(r.lookup(&d, &id).unwrap(), b"first");
    r.seal_release(&d, &lock, &admin, &h2).unwrap();
    let second = r.activate_release(&d, &lock, &admin, 1, &h2).unwrap();
    assert_eq!(second.revision, 2);
    assert_eq!(r.lookup(&d, &id).unwrap(), b"second");
    let third = r
        .activate_release(&d, &lock, &admin, 2, &first.hash)
        .unwrap();
    assert_eq!(third.revision, 3);
    assert_eq!(r.lookup(&d, &id).unwrap(), b"first");
    assert!(r.activate_release(&d, &lock, &admin, 1, &h2).is_err());
    r.revoke(&d, &lock, &admin, &AppPrecond::Present(first.hash.to_vec()))
        .unwrap();
    assert_eq!(r.active_info(&d, &id).unwrap().revision, 4);
    assert!(!r.active_info(&d, &id).unwrap().live);
    assert!(r.activate_release(&d, &lock, &admin, 4, &h2).is_err());
    r.register(&d, &lock, &admin, b"first".to_vec()).unwrap();
    assert_eq!(r.active_info(&d, &id).unwrap().revision, 5);
    assert!(r.activate_release(&d, &lock, &admin, 1, &h2).is_err());
    assert_eq!(r.list_ready(&d, &id).unwrap(), {
        let mut hashes = vec![first.hash, h2];
        hashes.sort();
        hashes
    });
    let older = store::encode_active(&r.active_info(&d, &id).unwrap());
    assert_eq!(
        store::decode_active(&older),
        Some(r.active_info(&d, &id).unwrap())
    );
}
