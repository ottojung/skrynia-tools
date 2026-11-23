//! Bounded-resource gate: wake hint queue quota fails closed.
mod common;
use common::*;
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;

#[test]
fn wake_hint_queue_is_bounded_and_fails_closed() {
    let d = Mem::default();
    subscribe(&d, WAKE);
    for i in 0..push::MAX_HINTS {
        put(&d, format!("thread-{i:04}-tes-v1").as_bytes(), b"v");
    }
    assert!(
        ops::put(
            &d,
            &NoLock,
            "d",
            WriteAuth::External(&caps().0),
            b"overflow-tes-v1",
            &AppPrecond::Absent,
            b"v",
        )
        .is_err(),
        "quota must reject mutations, not drop the wake"
    );
}
