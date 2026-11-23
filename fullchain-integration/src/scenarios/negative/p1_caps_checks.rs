//! Exact-key, grant and sender-capability negative assertions.
use super::p1_caps_report::report;
use super::p1_caps_support::{Caps, h};
use crate::{blb, call, idb, req, req_ver};
use webpush_sender::p1::HttpP1;

pub fn negatives(io: &HttpP1, c: &Caps, key: &[u8], sub: &[u8]) -> Vec<String> {
    let bad_grant = call(
        io,
        &req(16, &[&idb(&c.bad), &blb(key), &blb(sub), &blb(b"{}")]),
    )
    .unwrap_err();
    let sib = call(
        io,
        &req(
            16,
            &[&idb(&c.grant), &blb(b"other"), &blb(sub), &blb(b"{}")],
        ),
    )
    .unwrap_err();
    let bad_sender = call(
        io,
        &req(20, &[&idb(&c.bad_s), &idb(&h(&c.admin)), &blb(key)]),
    )
    .unwrap_err();
    let bad_fetch = call(
        io,
        &req(23, &[&idb(&c.bad_s), &idb(&h(&c.admin)), &blb(key)]),
    )
    .unwrap_err();
    let malformed = call(io, &[1u8, 20, 0xff]).unwrap_err();
    // Well-formed op20 body but wrong native P1 VERSION byte: must be rejected.
    let bad_version = call(
        io,
        &req_ver(2, 20, &[&idb(&c.sender), &idb(&h(&c.admin)), &blb(key)]),
    )
    .unwrap_err();
    report(
        bad_grant.0,
        sib.0,
        bad_sender.0,
        bad_fetch.0,
        malformed.0,
        bad_version.0,
    )
}
