//! Bounded-resource gate: the wake queue may never block data-plane writes.
//! Supersedes PR89's fail-closed quota assertion (that behaviour was the bug).
mod common;
use common::*;
use skrynia_native_service::registry::appdata::push;

#[test]
fn wake_queue_cannot_make_an_unrelated_namespace_write_busy() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    seed_sender(&d, &SENDER_A);
    let n = push::MAX_HINTS + 32;
    for i in 0..n {
        assert!(
            put_ok(&d, format!("thread-{i:04}-tes-v1").as_bytes()),
            "write {i} must commit even though wake state is durable"
        );
    }
    assert_eq!(hints(&d, WAKE), 1, "wake remains bounded after quota depth");
    assert_eq!(pending(&d), vec![n as u64]);
}
