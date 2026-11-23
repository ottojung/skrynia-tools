//! Seeds operator, grant and sender capabilities for the P1 negative cases.
use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use crate::chain::op_cap;
use sha2::{Digest, Sha256};
use std::path::Path;
use webpush_sender::p1::HttpP1;

fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub struct Caps {
    pub admin: [u8; 32],
    pub grant: [u8; 32],
}

pub fn install(io: &HttpP1, art: &Path) -> Caps {
    let admin = h(b"fc-admin-cap");
    let grant = h(b"fc-grant-cap");
    let sender = h(b"fc-sender-cap");
    let key = b"notice/alpha";
    let sub = b"sub-a";
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    must(
        io,
        &req(15, &[&idb(&op_cap()), &idb(&admin), &blb(&echo)]),
        "reg",
    );
    must(
        io,
        &req(18, &[&idb(&admin), &idb(&grant), &blb(key), &blb(sub)]),
        "grant",
    );
    must(
        io,
        &req(16, &[&idb(&grant), &blb(key), &blb(sub), &blb(b"{}")]),
        "sub",
    );
    must(
        io,
        &req(19, &[&idb(&admin), &idb(&sender), &blb(key)]),
        "sgrant",
    );
    Caps { admin, grant }
}
