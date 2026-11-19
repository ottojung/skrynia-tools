use super::*;
#[test]
fn reused_random_id_never_overwrites_existing_program() {
    let disk = NoList::default();
    let (status, id) = call(
        &disk,
        Request::Register {
            code: b"A".to_vec(),
        },
    );
    assert_eq!(status, STATUS_OK);
    assert_eq!(
        call(
            &disk,
            Request::Register {
                code: b"B".to_vec()
            }
        )
        .0,
        STATUS_ERR
    );
    assert_eq!(call(&disk, Request::Lookup { id }).1, b"A");
}
