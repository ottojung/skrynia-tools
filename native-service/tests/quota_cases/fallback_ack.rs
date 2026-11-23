//! A conditional ACK on the fallback must not hide a newer data mutation.
use super::sender_setup::full;
use super::support::{KEY, SENDER, list};
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes};

#[test]
fn fallback_dirty_generation_survives_superseded_read_and_ack() {
    let (d, w, ns) = full();
    let put = |expect, value| {
        ops::put(
            &d,
            &NoLock,
            "d",
            WriteAuth::External(&w),
            KEY,
            &expect,
            value,
        )
    };
    put(AppPrecond::Absent, b"first").unwrap();
    assert_eq!(
        list(&d, &w, KEY).len(),
        1,
        "the dirty slot supersedes the old full queue"
    );
    let g1 = *list(&d, &w, KEY).last().unwrap();
    assert_ne!(g1 & (1u64 << 63), 0, "fallback has a distinct wire seq");
    assert_eq!(
        push::hint_read(&d, &NoLock, "d", &SENDER, &ns, KEY, g1).unwrap(),
        vec![0u8; 32]
    );
    put(
        AppPrecond::Present(hash_bytes(b"first").to_vec()),
        b"second",
    )
    .unwrap();
    push::hint_ack(&d, &NoLock, "d", &SENDER, &ns, KEY, g1).unwrap();
    let g2 = *list(&d, &w, KEY).last().unwrap();
    assert_eq!(g2, g1 + 1);
    push::hint_ack(&d, &NoLock, "d", &SENDER, &ns, KEY, g2).unwrap();
    assert_eq!(list(&d, &w, KEY).len(), push::MAX_HINTS);
}
