//! Wake-only issuer cannot provision non-reserved wake channels.
use super::{Mem, wake_setup};
use skrynia_native_service::registry::appdata::push;
use skrynia_native_service::{NoLock, read_cap};

#[test]
fn wake_grant_rejects_an_arbitrary_data_key() {
    let (d, writer, _, _, _) = wake_setup();
    let denied = push::issue_grant::<Mem, NoLock>(
        &d,
        &NoLock,
        "d",
        &push::Kind::Wake,
        &[1u8; 32],
        &[55u8; 32],
        b"ordinary-key",
        &read_cap(&writer),
    );
    assert!(denied.is_err());
}
