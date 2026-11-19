use super::{R, handle, req};
use sha2::{Digest, Sha256};
#[test]
fn store_get_head_and_missing() {
    let b = b"hello".to_vec();
    let tag = format!("\"{}\"", hex::encode(Sha256::digest(&b)));
    let r = handle(&R(Some(b.clone())), &req("GET", "/store/a.txt"));
    assert_eq!(r.status, 200);
    assert_eq!(r.headers.get("etag"), Some(&tag));
    let h = handle(&R(Some(b)), &req("HEAD", "/store/a.txt"));
    assert!(h.status == 200 && h.body.is_empty());
    assert_eq!(handle(&R(None), &req("GET", "/store/a.txt")).status, 404);
}
