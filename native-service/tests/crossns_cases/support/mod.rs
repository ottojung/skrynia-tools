use super::common::Mem;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, hash_bytes, read_cap};
pub fn wake_setup() -> (Mem, [u8; 32], [u8; 32], [u8; 32], [u8; 32]) {
    let d = Mem::default();
    let (ww, wa, wb, sg, sub) = ([1u8; 32], [2u8; 32], [3u8; 32], [4u8; 32], [5u8; 32]);
    let wk = b"skrynia-wake-v1";
    let sh = hash_bytes(&sub);
    push::issue_grant::<Mem, NoLock>(&d, &NoLock, "d", &push::Kind::Subscribe, &ww, &sg, wk, &sh)
        .unwrap();
    push::subscribe(&d, &NoLock, "d", &sg, wk, &sub, b"ep").unwrap();
    for (src, gc) in [(&wa, [6u8; 32]), (&wb, [7u8; 32])] {
        push::issue_grant::<Mem, NoLock>(
            &d,
            &NoLock,
            "d",
            &push::Kind::Wake,
            &ww,
            &gc,
            wk,
            &read_cap(src),
        )
        .unwrap();
    }
    (d, wa, wb, [6u8; 32], [7u8; 32])
}
mod grant_test;
mod hints;
mod sender_test;
mod wire_test;
pub use hints::hints;

pub fn iw(d: &Mem, ww: &[u8; 32], gc: &[u8; 32], wk: &[u8], src: &[u8; 32]) {
    push::issue_grant(
        d,
        &NoLock,
        "d",
        &push::Kind::Wake,
        ww,
        gc,
        wk,
        &read_cap(src),
    )
    .unwrap();
}

mod revoke;
pub use revoke::Revoke;
