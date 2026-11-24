//! Shared exact-key sender fixture with a completely full legacy hint queue.
use super::support::{KEY, SENDER, sub};
use crate::common::Mem;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, read_cap};
pub fn full() -> (Mem, [u8; 32], [u8; 32]) {
    let d = Mem::default();
    let w = [3u8; 32];
    let ns = read_cap(&w);
    sub(&d, &w, KEY);
    push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Sender,
        &w,
        &SENDER,
        KEY,
        &[0u8; 32],
    )
    .unwrap();
    for i in 0..push::MAX_HINTS {
        push::enqueue_hints(&d, "d", &ns, KEY, Some([i as u8; 32])).unwrap();
    }
    (d, w, ns)
}
