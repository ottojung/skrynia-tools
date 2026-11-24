//! Shared P1 bootstrap for negative push-delivery scenarios.
use crate::{blb, idb, must, op_cap, req};
use sha2::{Digest, Sha256};
use webpush_sender::p1::HttpP1;

pub fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}
pub fn setup(io: &HttpP1, echo: &[u8], key: &[u8], sub_json: &[u8], sender: &[u8; 32]) {
    let admin = h(b"a");
    let grant = h(b"g");
    must(
        io,
        &req(15, &[&idb(&op_cap()), &idb(&admin), &blb(echo)]),
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
    must(io, &req(19, &[&idb(&admin), &idb(sender), &blb(key)]), "sg");
}
