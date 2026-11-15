use super::{Release, record};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn immutable_metadata_cannot_be_overwritten() {
    let root = std::env::temp_dir().join(format!("skrynia-record-{:032x}", rand::random::<u128>()));
    fs::create_dir(&root).unwrap();
    let entry = Release {
        id: "0123456789ab-0123456789abcdef".into(),
        name: "hello".into(),
        repo: "https://github.com/ottojung/demo".into(),
        commit: "0123456789abcdef0123456789abcdef01234567".into(),
        subdir: ".".into(),
        builder_image: "builder".into(),
        published_unix_ms: Some(123),
        content_sha256: None,
    };
    record::save(&root, &entry).unwrap();
    let path = root.join(format!("{}.json", entry.id));
    let original = fs::read(&path).unwrap();
    assert!(record::save(&root, &entry).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let mut oversized = entry;
    oversized.id = "0123456789ab-0123456789abcdee".into();
    oversized.builder_image = "x".repeat(20000);
    assert!(record::save(&root, &oversized).is_err());
    assert!(!root.join(format!("{}.json", oversized.id)).exists());
    fs::remove_dir_all(root).unwrap();
}
