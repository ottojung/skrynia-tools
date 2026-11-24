//! Wake-slot observation helpers (issue 5z confine).
use super::crosskey::{SEND, WCAP};
use super::{Mem, WAKE, caps};
use skrynia_native_service::NoLock;
use skrynia_native_service::cap::Disk;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{hash_bytes, read_cap};

pub fn sns() -> [u8; 32] {
    read_cap(&caps().0)
}
fn wns() -> [u8; 32] {
    read_cap(&WCAP)
}

pub fn sub_exact(d: &Mem, key: &[u8]) {
    let (g, s) = ([46u8; 32], [47u8; 32]);
    push::issue_grant::<Mem, NoLock>(
        d,
        &NoLock,
        "d",
        &push::Kind::Subscribe,
        &caps().0,
        &g,
        key,
        &hash_bytes(&s),
    )
    .unwrap();
    push::subscribe(d, &NoLock, "d", &g, key, &s, b"ep").unwrap();
}

pub fn gens(d: &Mem) -> Vec<u64> {
    let w = push::hint_list(d, &NoLock, "d", &SEND, &wns(), WAKE).unwrap();
    let n = u32::from_le_bytes(w[..4].try_into().unwrap()) as usize;
    (0..n)
        .map(|i| u64::from_le_bytes(w[4 + i * 8..12 + i * 8].try_into().unwrap()))
        .collect()
}

pub fn slot(d: &Mem, ns: &[u8; 32], key: &[u8]) -> bool {
    d.read_all(&push::wake_slot_path("d", ns, key))
        .unwrap()
        .is_some()
}
