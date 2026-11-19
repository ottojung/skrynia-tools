use super::*;
#[test]
fn every_root_request_targets_one_program_without_listing() {
    let disk = NoList::default();
    let (status, id) = call(
        &disk,
        Request::Register {
            code: b"A".to_vec(),
        },
    );
    assert_eq!(status, STATUS_OK);
    assert_eq!(disk.reads(), 1, "collision check reads one candidate only");
    assert_eq!(
        call(&disk, Request::Lookup { id: id.clone() }),
        (STATUS_OK, b"A".to_vec())
    );
    assert_eq!(
        call(
            &disk,
            Request::Invoke {
                id: id.clone(),
                input: b"!".to_vec()
            }
        ),
        (STATUS_OK, b"A!".to_vec())
    );
    assert_eq!(
        call(
            &disk,
            Request::Set {
                id: id.clone(),
                code: b"B".to_vec()
            }
        )
        .0,
        STATUS_OK
    );
    assert_eq!(
        call(
            &disk,
            Request::Invoke {
                id: id.clone(),
                input: b"?".to_vec()
            }
        ),
        (STATUS_OK, b"B?".to_vec())
    );
    assert_eq!(call(&disk, Request::Revoke { id: id.clone() }).0, STATUS_OK);
    assert_eq!(call(&disk, Request::Lookup { id }).0, STATUS_ERR);
}
