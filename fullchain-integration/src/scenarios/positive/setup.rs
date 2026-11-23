use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use super::ctx::Ctx;
use sha2::{Digest, Sha256};

fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn bootstrap(c: &mut Ctx) {
    let op = crate::chain::op_cap();
    let admin = h(b"fc-admin-cap");
    let grant = h(b"fc-grant-cap");
    let sub = b"sub-alpha";
    c.ns = h(&admin);
    c.sender_cap = h(b"fc-sender-cap");
    let echo = std::fs::read(c.art.join("echo_dc28e490.wasm")).expect("echo");
    must(
        &c.io,
        &req(15, &[&idb(&op), &idb(&admin), &blb(&echo)]),
        "register",
    );
    must(
        &c.io,
        &req(18, &[&idb(&admin), &idb(&grant), &blb(&c.key), &blb(sub)]),
        "grant",
    );
    must(
        &c.io,
        &req(
            16,
            &[&idb(&grant), &blb(&c.key), &blb(sub), &blb(&c.sub_json)],
        ),
        "subscribe",
    );
    must(
        &c.io,
        &req(6, &[&idb(&admin), &blb(&c.key), &[0u8], &blb(b"v1")]),
        "put",
    );
    must(
        &c.io,
        &req(19, &[&idb(&admin), &idb(&c.sender_cap), &blb(&c.key)]),
        "sender-grant",
    );
    c.ns = h(&admin); // namespace == sha256(admin) for push ops
}
