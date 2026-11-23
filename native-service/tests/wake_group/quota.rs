//! Wake state stays O(1) past the ordinary hint queue depth.
use crate::common::*;
use skrynia_native_service::registry::appdata::push;

#[test]
fn wake_queue_cannot_make_an_unrelated_namespace_write_busy() {
    let d = seeded();
    let n = push::MAX_HINTS + 32;
    for i in 0..n {
        assert!(
            put_ok(&d, format!("thread-{i:04}-tes-v1").as_bytes()),
            "write {i} must commit even past the hint queue limit"
        );
    }
    assert_eq!(list(&d), vec![n as u64]);
    assert_eq!(hints(&d, WAKE), 0, "no per-generation files to accumulate");
}
