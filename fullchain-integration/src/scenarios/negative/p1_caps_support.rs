//! Shared cap derivation and P1 bootstrap for negative capability cases.
use crate::{blb, idb, must, op_cap, req};
use sha2::{Digest, Sha256};
use webpush_sender::p1::HttpP1;

pub fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub struct Caps {
    pub admin: [u8; 32],
    pub grant: [u8; 32],
    pub bad: [u8; 32],
    pub sender: [u8; 32],
    pub bad_s: [u8; 32],
}
pub fn caps() -> Caps {
    Caps {
        admin: h(b"fc-admin-cap"),
        grant: h(b"fc-grant-cap"),
        bad: h(b"fc-bad-grant"),
        sender: h(b"fc-sender-cap"),
        bad_s: h(b"fc-bad-sender"),
    }
}
pub fn setup(io: &HttpP1, echo: &[u8], c: &Caps, key: &[u8], sub: &[u8]) {
    must(
        io,
        &req(15, &[&idb(&op_cap()), &idb(&c.admin), &blb(echo)]),
        "reg",
    );
    must(
        io,
        &req(18, &[&idb(&c.admin), &idb(&c.grant), &blb(key), &blb(sub)]),
        "grant",
    );
    must(
        io,
        &req(16, &[&idb(&c.grant), &blb(key), &blb(sub), &blb(b"{}")]),
        "sub",
    );
    must(
        io,
        &req(19, &[&idb(&c.admin), &idb(&c.sender), &blb(key)]),
        "sgrant",
    );
}
