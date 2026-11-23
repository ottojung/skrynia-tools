//! Seeds the P1 capabilities used by the negative push scenarios.
use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use crate::chain::op_cap;
use sha2::{Digest, Sha256};
use std::path::Path;
use webpush_sender::p1::HttpP1;

fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn install(io: &HttpP1, art: &Path, sub_json: &[u8]) -> ([u8; 32], [u8; 32]) {
    let admin = h(b"a");
    let grant = h(b"g");
    let sender = h(b"s");
    let key = b"k";
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    must(
        io,
        &req(15, &[&idb(&op_cap()), &idb(&admin), &blb(&echo)]),
        "r",
    );
    must(
        io,
        &req(18, &[&idb(&admin), &idb(&grant), &blb(key), &blb(b"sub")]),
        "g",
    );
    must(
        io,
        &req(16, &[&idb(&grant), &blb(key), &blb(b"sub"), &blb(sub_json)]),
        "s",
    );
    must(
        io,
        &req(6, &[&idb(&admin), &blb(key), &[0u8], &blb(b"v")]),
        "p",
    );
    must(
        io,
        &req(19, &[&idb(&admin), &idb(&sender), &blb(key)]),
        "sg",
    );
    (admin, sender)
}
