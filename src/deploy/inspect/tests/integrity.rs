use super::*;

#[test]
fn explicit_integrity_check_detects_modified_artifacts_and_legacy_records() {
    let fixture = Fixture::new();
    let mut release_record = fixture.add(1);
    assert!(
        super::super::verify(&fixture.root, "hello", &release_record.id)
            .unwrap()
            .verified
            .is_none()
    );
    let tree = fixture.root.join("releases/hello").join(&release_record.id);
    release_record.content_sha256 = Some(files::digest_tree(&tree).unwrap());
    let record = fixture
        .root
        .join("metadata/hello")
        .join(format!("{}.json", release_record.id));
    fs::write(record, serde_json::to_vec(&release_record).unwrap()).unwrap();
    let checked = super::super::verify(&fixture.root, "hello", &release_record.id).unwrap();
    assert_eq!(checked.verified, Some(true));
    fs::write(tree.join("index.html"), b"damaged").unwrap();
    let checked = super::super::verify(&fixture.root, "hello", &release_record.id).unwrap();
    assert_eq!(checked.verified, Some(false));
}
