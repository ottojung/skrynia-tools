//! Issue subscribe + sender grants, store one subscription, enqueue one hint.
use super::{MemDisk, caps, push};

pub(crate) fn seed_sender(disk: &MemDisk, sender: &[u8], key: &[u8]) {
    let (writer, grant, subscriber) = caps();
    push::issue_grant::<MemDisk, ()>(
        disk,
        "d",
        &push::Kind::Subscribe,
        &writer,
        &grant,
        key,
        &crate::hash_bytes(&subscriber),
    )
    .unwrap();
    push::subscribe(
        disk,
        &crate::NoLock,
        "d",
        &grant,
        key,
        &subscriber,
        b"endpoint",
    )
    .unwrap();
    push::issue_grant::<MemDisk, ()>(
        disk,
        "d",
        &push::Kind::Sender,
        &writer,
        &sender,
        key,
        &[0u8; 32],
    )
    .unwrap();
    push::enqueue_hints(disk, "d", &crate::read_cap(&writer), key, Some([5u8; 32])).unwrap();
}
