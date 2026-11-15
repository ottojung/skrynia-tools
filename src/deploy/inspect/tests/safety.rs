use super::*;
#[cfg(unix)]
use std::os::unix::fs::symlink;
#[test]
fn rollback_rejects_invalid_identifiers_and_oversized_metadata() {
    let fixture = Fixture::new();
    let target = fixture.add(1);
    fixture.active(&target.id);
    assert!(rollback(&fixture.root, "hello", "../../outside").is_err());
    let record = fixture
        .root
        .join("metadata/hello")
        .join(format!("{}.json", target.id));
    fs::write(record, vec![b' '; 16385]).unwrap();
    assert!(rollback(&fixture.root, "hello", &target.id).is_err());
    assert!(
        fs::read_link(fixture.root.join("apps/hello"))
            .unwrap()
            .to_string_lossy()
            .ends_with(&target.id)
    );
}
#[cfg(unix)]
#[test]
fn symlinked_release_metadata_is_rejected() {
    let fixture = Fixture::new();
    let target = fixture.add(1);
    let record = fixture
        .root
        .join("metadata/hello")
        .join(format!("{}.json", target.id));
    fs::remove_file(&record).unwrap();
    symlink("/etc/passwd", &record).unwrap();
    assert!(release(&fixture.root, "hello", &target.id).is_err());
}
