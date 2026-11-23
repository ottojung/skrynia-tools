//! Verify real P1 wire authorization and conditional sender ACK.
use super::{ctx::Ctx, wire as w};
use crate::{blb, call, idb, must, req, u64b};
pub fn verify(c: &Ctx) {
    w::write(c, &w::a(), &w::grant_a(), b"traveller-a", b"a");
    assert_eq!(w::list(c), [1]);
    assert_eq!(must(&c.io, &req(21, &[
        &idb(&w::sender()), &idb(&w::ns()), &blb(w::KEY), &u64b(1),
    ]), "read g1"), [0u8; 32]);
    w::write(c, &w::b(), &w::grant_b(), b"traveller-b", b"b");
    w::ack(c, 1);
    assert_eq!(w::list(c), [2]);
    let foreign = req(25, &[
        &idb(&w::b()), &blb(b"foreign"), &[0], &blb(b"v"),
        &idb(&w::grant_a()), &blb(w::KEY),
    ]);
    assert_eq!(call(&c.io, &foreign).unwrap_err().0, 9);
    let wrong = req(25, &[
        &idb(&w::a()), &blb(b"wrong"), &[0], &blb(b"v"),
        &idb(&w::grant_a()), &blb(b"another-exact-key"),
    ]);
    assert!(call(&c.io, &wrong).is_err());
    assert_eq!(w::list(c), [2]);
}
