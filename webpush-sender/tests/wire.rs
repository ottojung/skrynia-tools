mod common;

use common::{CAP, FakeP1, KEY, NS, res};
use webpush_sender::Error;
use webpush_sender::p1::P1Transport;
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
    assert_eq!((req[0], req[1]), (1, 23));
    assert_eq!(&req[2..4], &32u16.to_le_bytes());
    assert_eq!(&req[4..36], &CAP);
    assert_eq!(&req[36..38], &32u16.to_le_bytes());
    assert_eq!(&req[38..70], &NS);
    let n = u32::from_le_bytes(req[70..74].try_into().unwrap()) as usize;
    assert_eq!(&req[74..74 + n], KEY);
    assert_eq!(hint_list(&CAP, &NS, KEY).unwrap()[1], 20);
    assert_eq!(hint_read(&CAP, &NS, KEY, 7).unwrap()[1], 21);
    assert_eq!(hint_ack(&CAP, &NS, KEY, 7).unwrap()[1], 22);
    let p1 = FakeP1::new(vec![1], vec![]);
    assert!(p1.call(&[]).is_err());
    assert!(p1.call(&[20]).is_err());
    assert!(p1.call(&req).is_ok());
}

#[test]
fn response_decode_is_fail_closed() {
    assert_eq!(decode(&res(0, b"ok")).unwrap(), b"ok");
    assert_eq!(decode(&res(1, &[9])), Err(Error::P1(9)));
    assert_eq!(decode(&res(1, &[10, 0])), Err(Error::P1(10)));
    assert_eq!(decode(&[2, 0, 0, 0, 0, 0]), Err(Error::Wire("version")));
    assert_eq!(decode(&[1, 0, 9, 0, 0, 0]), Err(Error::Wire("len")));
    assert_eq!(decode(&[1]), Err(Error::Wire("short")));
}

#[test]
fn file_ledger_store_rejects_oversized_and_enforces_mode() {
    common::ledger_hard::oversized_and_nonfile_rejected();
    common::ledger_hard::writes_private_mode();
    common::ledger_tmp::stale_tmp_and_symlink_safe();
}
