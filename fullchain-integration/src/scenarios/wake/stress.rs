//! One subscriber's sender stays offline during >1024 real WASM writes.
use super::{ctx::Ctx, wire as w};
use crate::{blb, idb, must, req};
pub fn offline(c: &Ctx) {
    for i in 0..1040 {
        let (cap, grant) = if i % 2 == 0 {
            (w::a(), w::grant_a())
        } else {
            (w::b(), w::grant_b())
        };
        let key = format!("thread-{i:04}-a");
        w::write(c, &cap, &grant, key.as_bytes(), b"v");
    }
    assert_eq!(w::list(c), [1042]);
    w::ack(c, 1042);
    assert!(w::list(c).is_empty());
    must(&c.io, &req(26, &[
        &idb(&w::a()), &blb(b"traveller-a"), &w::h(b"a"),
        &idb(&w::grant_a()), &blb(w::KEY),
    ]), "delete wake");
    assert_eq!(w::list(c), [1043]);
}
