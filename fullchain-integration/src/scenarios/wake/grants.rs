//! One exact recipient key, two independently authorized source namespaces.
use super::{ctx::Ctx, wire as w};
use crate::{blb, idb, must, req};
pub fn provision(c: &Ctx) {
    let sub = b"disposable-crossns";
    let grant = w::h(b"wake-subscribe");
    must(&c.io, &req(18, &[
        &idb(&w::owner()), &idb(&grant), &blb(w::KEY), &blb(sub),
    ]), "wake subscribe grant");
    must(&c.io, &req(16, &[
        &idb(&grant), &blb(w::KEY), &blb(sub), &blb(&c.sub_json),
    ]), "wake subscribe");
    must(&c.io, &req(19, &[
        &idb(&w::owner()), &idb(&w::sender()), &blb(w::KEY),
    ]), "wake sender scope");
    for (src, g) in [(w::a(), w::grant_a()), (w::b(), w::grant_b())] {
        must(&c.io, &req(24, &[
            &idb(&w::owner()), &idb(&g), &blb(w::KEY), &idb(&w::h(&src)),
        ]), "wake-only grant");
    }
}
