use super::super::p1::must;
use super::super::wire::{blb, idb, req};
use super::ctx::Ctx;
use super::pump;
use sha2::{Digest, Sha256};
fn h(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

pub fn restart(c: &mut Ctx) -> Vec<String> {
    let admin = h(b"fc-admin-cap");
    let prev = h(b"v1");
    must(
        &c.io,
        &req(6, &[&idb(&admin), &blb(&c.key), &[1u8], &prev, &blb(b"v2")]),
        "put2",
    );
    let (rep, sent) = pump::deliver(c);
    assert_eq!((rep.hints, rep.delivered, rep.acked, sent), (1, 1, 1, 1));
    let (rep2, sent2) = pump::deliver_again(c);
    assert_eq!(
        (rep2.hints, rep2.delivered, rep2.acked, sent2),
        (0, 0, 0, 0)
    );
    vec!["restart_no_duplicate=true".into()]
}

pub fn revoke(c: &mut Ctx) -> Vec<String> {
    let grant = h(b"fc-grant-cap");
    let sub = b"sub-alpha";
    must(
        &c.io,
        &req(17, &[&idb(&grant), &blb(&c.key), &blb(sub)]),
        "unsub",
    );
    let admin = h(b"fc-admin-cap");
    let prev = h(b"v2");
    must(
        &c.io,
        &req(6, &[&idb(&admin), &blb(&c.key), &[1u8], &prev, &blb(b"v3")]),
        "put3",
    );
    let (rep, sent) = pump::deliver(c);
    assert_eq!(rep.delivered, 0);
    assert_eq!(sent, 0);
    vec!["revoke_stops_delivery=true".into()]
}
