use super::{changes::Changes, types::Artifact};
use std::collections::BTreeMap;
#[test]
fn classifies_added_removed_modified_and_executable_changes() {
    let plain = Artifact {
        kind: "file",
        sha256: Some("a".into()),
        executable: Some(false),
    };
    let exec = Artifact {
        kind: "file",
        sha256: Some("a".into()),
        executable: Some(true),
    };
    let folder = Artifact {
        kind: "directory",
        sha256: None,
        executable: None,
    };
    let before = BTreeMap::from([
        ("old".into(), plain.clone()),
        ("same".into(), folder.clone()),
        ("mode".into(), plain.clone()),
    ]);
    let after = BTreeMap::from([
        ("new".into(), plain),
        ("same".into(), folder),
        ("mode".into(), exec),
    ]);
    let identical = Changes::between(&before, &before);
    assert!(identical.added.is_empty() && identical.removed.is_empty());
    assert!(identical.modified.is_empty());
    let changes = Changes::between(&before, &after);
    assert_eq!(changes.added.len(), 1);
    assert_eq!(changes.added[0].path, "new");
    assert_eq!(changes.removed.len(), 1);
    assert_eq!(changes.removed[0].path, "old");
    assert_eq!(changes.modified.len(), 1);
    assert_eq!(changes.modified[0].path, "mode");
}
