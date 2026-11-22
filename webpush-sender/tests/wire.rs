mod common;

use common::{CAP, KEY, NS, res, scan};
use std::path::Path;
use webpush_sender::Error;
use webpush_sender::p1::{
    OP_HINT_ACK, OP_HINT_LIST, OP_HINT_READ, OP_SUB_FETCH, decode, hint_ack, hint_list, hint_read,
    sub_fetch,
};

#[test]
fn sender_opcodes_match_pr65() {
    assert_eq!(
        (OP_HINT_LIST, OP_HINT_READ, OP_HINT_ACK, OP_SUB_FETCH),
        (20, 21, 22, 23)
    );
    let req = sub_fetch(&CAP, &NS, KEY).unwrap();
    assert_eq!(req[0], 23);
    assert_eq!(&req[1..3], &32u16.to_le_bytes());
    assert_eq!(&req[3..35], &CAP);
    assert_eq!(&req[35..37], &32u16.to_le_bytes());
    assert_eq!(&req[37..69], &NS);
    let n = u32::from_le_bytes(req[69..73].try_into().unwrap()) as usize;
    assert_eq!(&req[73..73 + n], KEY);
    assert_eq!(hint_list(&CAP, &NS, KEY).unwrap()[0], 20);
    assert_eq!(hint_read(&CAP, &NS, KEY, 7).unwrap()[0], 21);
    assert_eq!(hint_ack(&CAP, &NS, KEY, 7).unwrap()[0], 22);
}

#[test]
fn response_decode_is_fail_closed() {
    assert_eq!(decode(&res(0, b"ok")).unwrap(), b"ok");
    assert_eq!(decode(&res(1, &[9])), Err(Error::P1(9)));
    assert_eq!(decode(&[2, 0, 0, 0, 0, 0]), Err(Error::Wire("version")));
    assert_eq!(decode(&[1, 0, 9, 0, 0, 0]), Err(Error::Wire("len")));
    assert_eq!(decode(&[1]), Err(Error::Wire("short")));
}

#[test]
fn no_legacy_v2_routes_in_source() {
    let bad = [
        "_skrynia/push",
        "push/subscriptions",
        "configure-push-rules",
    ];
    let mut seen = Vec::new();
    scan(Path::new("src"), &bad, &mut seen);
    assert!(seen.is_empty(), "legacy routes: {seen:?}");
}
