use super::support::fixture;
use skrynia_native_service::{cap::Disk, store, store_io, store_io::StoreErr};

#[test]
fn wrong_owner_corrupt_blob_or_forged_ready_fails_closed() {
    let (r, d, l, admin, id) = fixture();
    assert!(r.stage_release(&d, &l, &[10u8; 32], b"other").is_err());
    let hash = r.stage_release(&d, &l, &admin, b"new").unwrap();
    let ready = store::ready_path("reg", &id, &hash).unwrap();
    d.write_atomic(&ready, b"forged").unwrap();
    assert!(r.seal_release(&d, &l, &admin, &hash).is_err());
    assert!(r.activate_release(&d, &l, &admin, 1, &hash).is_err());
    d.remove(&ready).unwrap();
    let blob = store::blob_path("reg", &hash).unwrap();
    d.write_atomic(&blob, b"corrupted").unwrap();
    let failed = r.seal_release(&d, &l, &admin, &hash);
    assert!(failed.is_err());
    assert!(
        r.lookup(&d, &id).is_ok(),
        "original active code must survive"
    );
    assert_eq!(r.active_info(&d, &id).unwrap().revision, 1);
    assert!(matches!(
        store_io::read_blob(&d, "reg", &hash),
        Err(StoreErr::BadRecord)
    ));
}
