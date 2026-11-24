//! Cross-namespace wake seeding helpers (issue 5z confine).
use super::{Mem, WAKE, caps};
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{hash_bytes, read_cap};

pub const A: &[u8] = b"traveller-tes-v1";
pub const B: &[u8] = b"traveller-vitalik-v1";
pub const WCAP: [u8; 32] = [41u8; 32];
pub const SUBG: [u8; 32] = [42u8; 32];
pub const SUB: [u8; 32] = [43u8; 32];
pub const SEND: [u8; 32] = [44u8; 32];
pub const WG: [u8; 32] = [45u8; 32];

fn grant(d: &Mem, k: &push::Kind, wc: &[u8], g: &[u8], key: &[u8], s: &[u8]) {
    push::issue_grant::<Mem, NoLock>(d, &NoLock, "d", k, wc, g, key, s).unwrap();
}

pub fn seed(d: &Mem) {
    grant(
        d,
        &push::Kind::Subscribe,
        &WCAP,
        &SUBG,
        WAKE,
        &hash_bytes(&SUB),
    );
    push::subscribe(d, &NoLock, "d", &SUBG, WAKE, &SUB, b"endpoint").unwrap();
    grant(d, &push::Kind::Sender, &WCAP, &SEND, WAKE, &[0u8; 32]);
    grant(d, &push::Kind::Wake, &WCAP, &WG, WAKE, &read_cap(&caps().0));
}

pub fn put(d: &Mem, key: &[u8], cap: &[u8; 32]) -> bool {
    ops::put_wake(
        d,
        &NoLock,
        "d",
        WriteAuth::External(cap),
        key,
        &AppPrecond::Absent,
        b"v",
        &WG,
        WAKE,
    )
    .is_ok()
}
