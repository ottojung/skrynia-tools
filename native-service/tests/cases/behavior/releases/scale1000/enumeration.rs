use super::support::{N, setup, sorted};
use crate::cases::behavior::releases::protocol_helpers::{call, hashes};
use skrynia_native_service::wire::{AdminRequest, Request, STATUS_OK};

#[test]
fn enumeration_is_exact_complete_and_ordered_at_one_thousand() {
    let (reg, d, lock, _a, iid, expected) = setup();
    let all = sorted(&expected);
    assert_eq!(all.len(), N + 1);
    assert_eq!(reg.list_ready(&d, &iid).unwrap(), all);
    let mut cursor = None;
    let mut seen: Vec<[u8; 32]> = Vec::new();
    let mut pages = 0;
    loop {
        let res = call(
            &reg,
            &d,
            &lock,
            Request::Admin(AdminRequest::ListVersions {
                invoke_id: iid.clone(),
                after: cursor,
                limit: 256,
            }),
        );
        assert_eq!(res.status, STATUS_OK);
        let chunk = hashes(&res.body);
        if chunk.is_empty() {
            break;
        }
        pages += 1;
        seen.extend(chunk);
        cursor = Some(seen.last().unwrap().to_vec());
    }
    assert_eq!(pages, 4);
    assert_eq!(seen, all);
}
