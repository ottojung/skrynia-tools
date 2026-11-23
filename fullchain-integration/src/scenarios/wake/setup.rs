//! Bootstrap ordinary P1 only; no kernel/P0 app-specific authority.
use super::{ctx::Ctx, wire};
use crate::{blb, idb, must, op_cap, req};
use std::path::Path;
pub fn register(c: &Ctx, art: &Path) {
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    for cap in [wire::owner(), wire::a(), wire::b()] {
        must(&c.io, &req(15, &[
            &idb(&op_cap()), &idb(&cap), &blb(&echo),
        ]), "register independent source");
    }
}
