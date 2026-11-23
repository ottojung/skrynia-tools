use super::super::p1::{call, must};
use super::super::wire::{blb, idb, req};
use crate::chain::{EdgeProc, Kernel, op_cap, transport};
use sha2::{Digest, Sha256};
fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn run(art: &std::path::Path, tmp: &std::path::Path) -> Vec<String> {
    let k = Kernel::start(art, &tmp.join("k")).expect("kernel");
    let e = EdgeProc::start(art, &tmp.join("e"), k.port, &k.token).expect("edge");
    let io = transport(&e);
    let admin = h(b"fc-admin-cap");
    let grant = h(b"fc-grant-cap");
    let bad = h(b"fc-bad-grant");
    let sender = h(b"fc-sender-cap");
    let bad_s = h(b"fc-bad-sender");
    let sub = b"sub-a";
    let key = b"notice/alpha";
    let echo = std::fs::read(art.join("echo_dc28e490.wasm")).unwrap();
    must(
        &io,
        &req(15, &[&idb(&op_cap()), &idb(&admin), &blb(&echo)]),
        "reg",
    );
    must(
        &io,
        &req(18, &[&idb(&admin), &idb(&grant), &blb(key), &blb(sub)]),
        "grant",
    );
    must(
        &io,
        &req(16, &[&idb(&grant), &blb(key), &blb(sub), &blb(b"{}")]),
        "sub",
    );
    must(
        &io,
        &req(19, &[&idb(&admin), &idb(&sender), &blb(key)]),
        "sgrant",
    );
    let bad_grant = call(
        &io,
        &req(16, &[&idb(&bad), &blb(key), &blb(sub), &blb(b"{}")]),
    )
    .unwrap_err();
    let sib = call(
        &io,
        &req(16, &[&idb(&grant), &blb(b"other"), &blb(sub), &blb(b"{}")]),
    )
    .unwrap_err();
    let bad_sender = call(&io, &req(20, &[&idb(&bad_s), &idb(&h(&admin)), &blb(key)])).unwrap_err();
    let bad_fetch = call(&io, &req(23, &[&idb(&bad_s), &idb(&h(&admin)), &blb(key)])).unwrap_err();
    let malformed = call(&io, &[1u8, 20, 0xff]).unwrap_err();
    assert_eq!(bad_grant.0, 9);
    assert_eq!(sib.0, 9);
    assert_eq!(bad_sender.0, 9);
    assert_eq!(bad_fetch.0, 9);
    assert!(matches!(malformed.0, 1 | 254 | 255));
    vec![
        "neg_wrong_grant=9".into(),
        "neg_sibling_key=9".into(),
        "neg_bad_sender=9".into(),
        "neg_bad_fetch=9".into(),
        format!("neg_malformed={}", malformed.0),
    ]
}
