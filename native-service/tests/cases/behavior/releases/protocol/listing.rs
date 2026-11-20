use super::*;
use skrynia_native_service::wire::AdminRequest;

#[test]
fn version_list_pages_without_admin_secret() {
    let (r, d, l, a, id) = fixture();
    let programs = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ListPrograms {
            after: None,
            limit: 1,
        }),
    );
    assert_eq!(programs.status, STATUS_OK);
    let iid: [u8; 32] = id.clone().try_into().unwrap();
    assert_eq!(hashes(&programs.body), vec![iid]);
    let first = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: id.clone(),
            after: None,
            limit: 1,
        }),
    );
    assert_eq!(hashes(&first.body).len(), 1);
    let initial = hashes(&first.body)[0];
    let hash = r.stage_release(&d, &l, &a, b"next").unwrap();
    r.seal_release(&d, &l, &a, &hash).unwrap();
    let later = call(
        &r,
        &d,
        &l,
        Request::Admin(AdminRequest::ListVersions {
            invoke_id: id,
            after: None,
            limit: 256,
        }),
    );
    assert_eq!(later.status, STATUS_OK);
    assert_eq!(hashes(&later.body).len(), 2);
    assert!(hashes(&later.body).contains(&initial));
    assert!(hashes(&later.body).contains(&hash));
}
