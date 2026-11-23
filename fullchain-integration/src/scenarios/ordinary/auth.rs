//! Only exact data key is subscribed; there is no reserved WAKE_KEY subscriber.
use super::{ctx::Ctx, wire as w};
use crate::{blb, idb, must, op_cap, req};
use std::path::Path;
pub fn setup(c: &Ctx, art: &Path) {
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    must(&c.io, &req(15, &[
        &idb(&op_cap()), &idb(&w::writer()), &blb(&echo),
    ]), "register ordinary");
    let sub_cap = w::h(b"ordinary-subgrant");
    let sub = b"ordinary-disposable-subscriber";
    must(&c.io, &req(18, &[
        &idb(&w::writer()), &idb(&sub_cap), &blb(w::KEY), &blb(sub),
    ]), "ordinary subscribe grant");
    must(&c.io, &req(16, &[
        &idb(&sub_cap), &blb(w::KEY), &blb(sub), &blb(&c.sub_json),
    ]), "ordinary exact subscribe");
    must(&c.io, &req(19, &[
        &idb(&w::writer()), &idb(&w::sender()), &blb(w::KEY),
    ]), "ordinary sender grant");
}
