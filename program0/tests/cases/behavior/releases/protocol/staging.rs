use super::*;

#[test]
fn unsealed_code_cannot_be_activated() {
    let (r, d, l, a, id) = fixture();
    let active = call(
        &r,
        &d,
        &l,
        Request::GetActive {
            invoke_id: id.clone(),
        },
    );
    assert_eq!(active.status, STATUS_OK);
    assert_eq!(store::decode_active(&active.body).unwrap().revision, 1);
    let staged = call(
        &r,
        &d,
        &l,
        Request::StageCode {
            admin_cap: a.clone(),
            code: b"second".to_vec(),
        },
    );
    assert_eq!(staged.status, STATUS_OK);
    let fail = call(
        &r,
        &d,
        &l,
        Request::ActivateCode {
            admin_cap: a.clone(),
            expected_revision: 1,
            target: staged.body.clone(),
        },
    );
    assert_eq!(fail.status, STATUS_ERR);
    let seal = call(
        &r,
        &d,
        &l,
        Request::SealCode {
            admin_cap: a,
            hash: staged.body,
        },
    );
    assert_eq!(seal.status, STATUS_OK);
    assert_eq!(r.lookup(&d, &id).unwrap(), b"first");
}
