//! Cross-namespace fixture: WAKE owner + sender, writer grant, data-key sub.
use super::support::{KEY, SENDER, WAKE_OWNER, sub};
use crate::common::Mem;
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, read_cap};

/// Durable disk with a cross-namespace wake grant and a full-source data sub.
pub fn crossns() -> (Mem, [u8; 32], [u8; 32]) {
    let d = Mem::default();
    sub(&d, &WAKE_OWNER, push::WAKE_KEY);
    push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Sender,
        &WAKE_OWNER,
        &SENDER,
        push::WAKE_KEY,
        &[0u8; 32],
    )
    .unwrap();
    let (writer, grant) = ([2u8; 32], [6u8; 32]);
    push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Wake,
        &WAKE_OWNER,
        &grant,
        push::WAKE_KEY,
        &read_cap(&writer),
    )
    .unwrap();
    sub(&d, &writer, KEY);
    (d, writer, grant)
}
