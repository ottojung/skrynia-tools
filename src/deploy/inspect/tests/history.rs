use super::*;
#[test]
fn versions_are_chronological_and_legacy_records_remain_usable() {
    let fixture = Fixture::new();
    let older = fixture.add(1);
    let newer = fixture.add(2);
    let old_path = fixture
        .root
        .join("metadata/hello")
        .join(format!("{}.json", older.id));
    let mut old: serde_json::Value = serde_json::from_slice(&fs::read(&old_path).unwrap()).unwrap();
    old.as_object_mut().unwrap().remove("published_unix_ms");
    fs::write(old_path, serde_json::to_vec(&old).unwrap()).unwrap();
    fixture.active(&newer.id);
    let history = releases(&fixture.root, "hello").unwrap();
    assert_eq!(history[0].id, newer.id);
    assert_eq!(history[1].published_unix_ms, None);
    assert_eq!(
        apps(&fixture.root).unwrap()[0].active.as_ref().unwrap().id,
        newer.id
    );
    assert!(
        release(&fixture.root, "hello", &older.id)
            .unwrap()
            .is_some()
    );
}
