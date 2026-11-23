//! The sender read old hints BEFORE the new commit, then ACKs them afterwards.
use super::sender_setup::full;
use super::support::{KEY, SENDER, list};
use skrynia_native_service::NoLock;
use skrynia_native_service::registry::appdata::{WriteAuth, ops, push};
use skrynia_native_service::wire::AppPrecond;

#[test]
fn late_write_must_survive_ack_of_an_already_delivered_full_hint_queue() {
    let (disk, writer, ns) = full();
    let delivered = list(&disk, &writer, KEY);
    assert_eq!(delivered.len(), push::MAX_HINTS);
    for seq in &delivered {
        push::hint_read(&disk, &NoLock, "d", &SENDER, &ns, KEY, *seq).unwrap();
    }
    ops::put(
        &disk,
        &NoLock,
        "d",
        WriteAuth::External(&writer),
        KEY,
        &AppPrecond::Absent,
        b"after-delivery",
    )
    .unwrap();
    for seq in delivered {
        push::hint_ack(&disk, &NoLock, "d", &SENDER, &ns, KEY, seq).unwrap();
    }
    assert!(
        !list(&disk, &writer, KEY).is_empty(),
        "the newer post-read mutation must retain an outstanding invalidation"
    );
}
