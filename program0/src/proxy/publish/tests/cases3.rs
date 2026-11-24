use super::{ERR_VALIDATION, queue, wire};

const PUBLIC: &[u8] = b"apps/hello/releases/r1/index.html";

#[test]
fn mediated_reply_has_status_and_data_words() {
    let reply = wire::accepted(0);
    assert_eq!(reply.len(), 8);
    assert_eq!(wire::accepted(6).len(), 8);
    assert_eq!(&reply[..4], &0i32.to_le_bytes());
    assert_eq!(&reply[4..], &0u32.to_le_bytes());
}

#[test]
fn child_request_is_queued_not_executed_in_callback_context() {
    let req = wire::request(wire::OP_PUBLISH, PUBLIC);
    assert_eq!(wire::dispatch(&req), wire::accepted(0));
    let pending = queue::drain();
    assert_eq!(pending.len(), 1);
    assert!(pending[0].0);
    assert_eq!(pending[0].1, PUBLIC);
    assert!(queue::drain().is_empty());
    let bad = wire::request(wire::OP_PUBLISH, b"security/program0.sha256");
    assert_eq!(&wire::dispatch(&bad)[..4], &ERR_VALIDATION.to_le_bytes());
    assert!(queue::drain().is_empty());
}
