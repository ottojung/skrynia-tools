//! Once the ordinary queue reaches 256 entries, only constant-sized wake
//! state advances; 1,000 more mutations cannot expand callback listing cost.
use super::sender_setup::full;
use super::support::{KEY, SENDER, list};
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;
use skrynia_native_service::{NoLock, hash_bytes};
#[test]
fn highwater_legacy_hint_budget_prevents_unbounded_scan() {
    let (d, w, ns) = full();
    let mut previous = None;
    for i in 0..1050 {
        let v = (i as u64).to_le_bytes();
        let expect = match previous {
            None => AppPrecond::Absent,
            Some(h) => AppPrecond::Present(h),
        };
        ops::put(&d, &NoLock, "d", WriteAuth::External(&w), KEY, &expect, &v).unwrap();
        previous = Some(hash_bytes(&v).to_vec());
    }
    let seqs = list(&d, &w, KEY);
    assert_eq!(seqs.len(), 1, "one latest coalesced fallback");
    assert_ne!(seqs[0] & (1u64 << 63), 0);
    push::hint_ack(&d, &NoLock, "d", &SENDER, &ns, KEY, seqs[0]).unwrap();
    assert_eq!(list(&d, &w, KEY).len(), push::MAX_HINTS);
}
